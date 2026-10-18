function elapsed(attempts, budget) {
  var total = 0;
  for (var i = 0; i < attempts.length; i++) {
    var allowed = Math.max(0, budget - total);
    total += Math.min(attempts[i], allowed);
  }
  return total;
}
var actual = elapsed([80, 80], 100);
console.log("two 80ms attempts, 100ms total budget: " + actual);
if (actual !== 100) throw new Error("R1 failed: expected 100, got " + actual);
if (elapsed([20, 30], 100) !== 50) throw new Error("short attempts must remain 50");
if (elapsed([], 100) !== 0) throw new Error("no attempts must consume zero");
console.log("R1: all three checks passed");
