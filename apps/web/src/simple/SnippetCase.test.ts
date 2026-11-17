import {describe,it,expect} from 'vitest';
import vectors from '../../../simple-contract/snippet-case-cases.json';
import {snippetCase,snippetCaseWords,snippetCaseVersion} from './SnippetCase';
describe('frozen Unicode snippet case policy',()=>{it('uses the declared table version',()=>expect(snippetCaseVersion).toBe(vectors.case_map));for(const c of vectors.cases)it(`${c.mode}: ${c.value}`,()=>expect(snippetCase(c.value,c.mode as 'upper'|'lower')).toBe(c.expected));for(const c of vectors.words)it('tokenizes by the same letter table',()=>expect(snippetCaseWords(c.value)).toEqual(c.expected))});
