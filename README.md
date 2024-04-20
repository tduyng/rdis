# Build Your Own Redis

[![progress-banner](https://backend.codecrafters.io/progress/redis/344bc7bf-df33-4d81-b622-4d80b7fbe38a)](https://app.codecrafters.io/users/tduyng?r=2qF)

Welcome to the "Build Your Own Redis" Challenge repository for Rust solutions!

## Overview

This repository serves as a my Rust solutions to the
["Build Your Own Redis" Challenge](https://codecrafters.io/challenges/redis).

You'll build a toy Redis clone that's capable of handling
basic commands like `PING`, `SET` and `GET`. Along the way we'll learn about
event loops, the Redis protocol and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

## Getting started

Note: This section is for stages 2 and beyond.

1. Ensure you have latest `cargo` installed locally
1. Run `./spawn_redis_server.sh` to run your Redis server, which is implemented
   in `src/main.rs`. This command compiles your Rust project, so it might be
   slow the first time you run it. Subsequent runs will be fast.
1. Commit your changes and run `git push origin master` to submit your solution
   to CodeCrafters. Test output will be streamed to your terminal.
