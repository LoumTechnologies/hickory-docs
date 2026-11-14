import {expect,it} from "vitest";
import fixture from "../../../simple-contract/coverage-projections.json";
import {coverageProjection,type CoverageProjectionInput} from "./CoverageProjection";
for(const test of fixture.cases)it(test.name,()=>{expect(coverageProjection(test.record as unknown as CoverageProjectionInput)).toEqual(test.expected);});
