-- `hickory check` was renamed to `hickory test` (no alias). The runs.kind
-- column stored the CLI's verb, so the stored value and its CHECK constraint
-- follow the rename. Existing 'check' rows are rewritten rather than kept as
-- a second spelling of the same thing: `RunKind` has one variant for this
-- kind of run, so the column must have one string for it too.
ALTER TABLE runs DROP CONSTRAINT runs_kind_check;

UPDATE runs SET kind = 'test' WHERE kind = 'check';

ALTER TABLE runs ADD CONSTRAINT runs_kind_check
    CHECK (kind IN ('run', 'test', 'agent'));
