"""Focused ownership checks; substitutes tool calls and never operates a simulator."""
from pathlib import Path
import importlib.util,io,json,subprocess,sys,tempfile,unittest
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1]
SEED='22222222-2222-2222-2222-222222222222'
OWNED='11111111-1111-1111-1111-111111111111'
RUNTIME='com.apple.CoreSimulator.SimRuntime.iOS-27-0'
class SimulatorOwnershipTests(unittest.TestCase):
 def load(self):
  spec=importlib.util.spec_from_file_location('owned_ios_runner',ROOT/'scripts/test-ios-parity.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
 def failure(self,temporary):
  runner=self.load();calls=[]
  def output(args,**kwargs):
   calls.append(args)
   if args[:4]==['xcrun','simctl','list','devices']:return json.dumps({'devices':{RUNTIME:[{'udid':SEED,'state':'Shutdown','isAvailable':True,'deviceTypeIdentifier':'owned-type'}]}}).encode()
   self.assertEqual(args[:3],['xcrun','simctl','create']);self.assertEqual(args[-2:],['owned-type',RUNTIME]);return OWNED+'\n'
  def execute(args,**kwargs):
   calls.append(args)
   if args[0]=='xcodegen':raise RuntimeError('owned failure before build')
   return subprocess.CompletedProcess(args,0)
  with tempfile.TemporaryDirectory()as folder,patch.object(runner,'ROOT',Path(folder)),patch.object(runner,'require_space'),patch.object(runner.subprocess,'check_output',side_effect=output),patch.object(runner.subprocess,'run',side_effect=execute),patch.object(sys,'argv',['runner','--device',SEED,'--boot','--build']+(['--temporary-device']if temporary else[])):
   with self.assertRaisesRegex(RuntimeError,'owned failure before build'):runner.main()
   receipt=json.loads((Path(folder)/'target/ios-parity/last-device-lifecycle.json').read_text());self.assertEqual(receipt['temporary_device'],temporary)
   return calls,receipt
 def test_owned_device_deleted_after_prebuild_failure_without_touching_seed(self):
  calls,receipt=self.failure(True);self.assertIn(['xcrun','simctl','delete',OWNED],calls);self.assertIn(['xcrun','simctl','shutdown',OWNED],calls);self.assertTrue(receipt['owned_device_deleted'])
  for call in calls:
   if call[:2]==['xcrun','simctl']and call[2]in ['boot','shutdown','delete']:self.assertNotIn(SEED,call[2:])
 def test_existing_seed_restored_without_any_delete(self):
  calls,_=self.failure(False);self.assertIn(['xcrun','simctl','shutdown',SEED],calls);self.assertFalse(any(call[:3]==['xcrun','simctl','delete']for call in calls))
 def test_a_preexisting_create_result_never_authorizes_boot_or_delete(self):
  runner=self.load();devices={'devices':{RUNTIME:[{'udid':SEED,'state':'Shutdown','isAvailable':True,'deviceTypeIdentifier':'owned-type'}]}}
  with tempfile.TemporaryDirectory()as folder,patch.object(runner,'ROOT',Path(folder)),patch.object(runner,'require_space'),patch.object(runner.subprocess,'check_output',side_effect=[json.dumps(devices).encode(),SEED+'\n']),patch.object(runner.subprocess,'run')as mutate,patch.object(sys,'argv',['runner','--device',SEED,'--temporary-device','--boot','--build']):
   with self.assertRaisesRegex(RuntimeError,'preexisting simulator identity'):runner.main()
   mutate.assert_not_called();receipt=json.loads((Path(folder)/'target/ios-parity/last-device-lifecycle.json').read_text());self.assertFalse(receipt['temporary_device']);self.assertFalse(receipt['booted_here'])
 def test_temporary_mode_without_explicit_boot_cannot_create_a_device(self):
  runner=self.load();devices={'devices':{RUNTIME:[{'udid':SEED,'state':'Shutdown','isAvailable':True,'deviceTypeIdentifier':'owned-type'}]}}
  with patch.object(runner,'require_space'),patch.object(runner.subprocess,'check_output',return_value=json.dumps(devices).encode())as read,patch.object(runner.subprocess,'run')as mutate,patch.object(sys,'argv',['runner','--device',SEED,'--temporary-device']),patch('sys.stderr',new=io.StringIO()):
   with self.assertRaises(SystemExit)as error:runner.main()
   self.assertEqual(error.exception.code,2);self.assertEqual(read.call_count,1);mutate.assert_not_called()
if __name__=='__main__':unittest.main()
