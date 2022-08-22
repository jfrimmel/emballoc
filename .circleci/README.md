# CI definitions for CircleCI

This directory contains all the necessary files for th Continuous Integration setup.
Most importantly, there is the [config.yml](config.yml)-file, which contains the core definitions of the CI.
It also contains a docker file for a custom container called `jfrimmel/miri`.
This container is a version of `rust`, which already contains a pre-setup `miri` environment.

To build the docker container, select the desired nightly version (which, of course, has to contain the `miri` component) and run the following command from the repository root (replace `2022-08-22` with the required version):

```bash
docker build -t jfrimmel/miri:nightly-2022-08-22 --build-arg nightly_version=2022-08-22 -f .circleci/miri.Dockerfile .circleci/
```
