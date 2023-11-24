---
sidebar_position: 103
---

# Migration guide for updating to new versions

<!-- TODO -->
:construction: This section has not been finished yet.

## General information

After installing/deploying the new version
[`opentalk-controller fix-acl`](acl.md#opentalk-controller-fix-acl-subcommand)
must be run in order to update ACLs to match the newest version whenever
new endpoints were added for already present resources. However, even if no
endpoints were added, simply running the command does no harm.
