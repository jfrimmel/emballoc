# CI definitions for CircleCI

This directory contains all the necessary files for th Continuous Integration setup.
Most importantly, there is the [config.yml](config.yml)-file, which contains the core definitions of the CI.
It also contains docker files for custom containers used during the CI run.

## Container for Miri

This container is a version of `rust`, which already contains a pre-setup `miri` environment.

To build the docker container, select the desired nightly version (which, of course, has to contain the `miri` component) and run the following command from the repository root (store the requested version in `$VERSION`):

```bash
VERSION=2022-08-22
docker build -t "jfrimmel/miri:nightly-$VERSION" --build-arg "nightly_version=$VERSION" -f .circleci/miri.Dockerfile .circleci/
docker push "jfrimmel/miri:nightly-$VERSION"
```

## Container for coverage testing

This container is a version of `rust`, which already contains a pre-installed coverage-testing tools.

To build the container use:

```bash
VERSION=2022-08-22
docker build -t "jfrimmel/coverage:nightly-$VERSION" --build-arg "nightly_version=$VERSION" -f .circleci/coverage.Dockerfile .circleci/
```
