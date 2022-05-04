# Concordium voting smart contract

This is the first voting contract.

The way to create a voting agenda is to initialize the contract with the title, description and proposals as parameters.

The function that gives voting rights is excluded in this version.

The code related to the ability to grant voting rights is commented out.


The current specifications are as follows.

- You can vote with any account.
- Each account has one vote.
- You can change the options until the voting is completed.


In this version you can do the following for testing:

- Even after the deadline has passed, you can vote if the data is not counted.
- Anyone can execute the aggregation method.
- Aggregation is possible even before the deadline.

## Installation
Clone concordium standard libraries.

```
git submodule update --init --recursive
```
