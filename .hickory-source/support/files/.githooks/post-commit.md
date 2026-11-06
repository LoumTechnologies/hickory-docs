<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# .githooks/post-commit

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `2ded29604bfe9dddf237f24fc685183b0c2a1149f2ea332c2721ec2d1f2ad59b`.

<book:entry path=".githooks/post-commit" executable="true">
#!/usr/bin/env bash
# Git waits for hooks: detach the blocking command and disconnect all terminal
# descriptors. Checks and Git operations never wait for one another.
root=$(git rev-parse --show-toplevel) || exit 0
mode=$(git config --get hickory.postCommitChecks || echo local)
case "$mode" in
  off) exit 0 ;;
  local) ;;
  local-ci) revision=$(git rev-parse --verify HEAD) || exit 0 ;;
  *) echo "Unknown hickory.postCommitChecks '$mode'; use just checks-hook local, local-ci or off" >&2; exit 1 ;;
esac
common=$(git rev-parse --path-format=absolute --git-common-dir) || exit 0
mkdir -p "$common/checks" || exit 0
# Clear Git's inherited index/worktree variables before the detached worker.
for name in $(git rev-parse --local-env-vars); do unset "$name"; done
cd "$root" || exit 0
if [ "$mode" = local-ci ]; then
  nohup just checks-submit "$revision" </dev/null >>"$common/checks/hook.log" 2>&1 &
else
  nohup just checks </dev/null >>"$common/checks/hook.log" 2>&1 &
fi
exit 0
</book:entry>

</book:doc>
