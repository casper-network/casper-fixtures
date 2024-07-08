# Fixture/2.0.0 migration testing example
- `migration-test-contract`: a primitive cep-18 like contract that allows someone to get a token or give someone else a token.
- `owner-contract`: a primitive contract that has a single entrypoint that calls into the `migration-test-contract` to gain a token.
- `tests`: tests

## pre-condor
versions of the contracts and the tests compatible with 1.5.6 (meaning before 2.0.0/condor)

## post-condor
versions of the contracts and tests that have been made to work on 2.0.0/condor

## migration test
1. In the pre-condor version looking at the `fixgure_gen.rs` file you can see how to set up a global state with the contracts deployed and some preliminary usage of the contract, then export the global state into lmdb
2. move the tests/fixtures directory from the pre-condor to the post-condor version
3. look inside the post-condor `tests/src/migration.rs` file to see how we load the lmdb global state and how we proceed after that. Take note that after loading the global state from lmdb, you also have to upgrade the chainspec (specific code inside `upgrade_v1_5_6_fixture_to_v2_0_0_ee` function).