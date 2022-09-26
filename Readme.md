# Rust Full GRPC Example

## Getting Started

1. Go to [rust site](https://www.rust-lang.org/tools/install) and follow the instructions.

    Note: on Windows, ensure you install the gnu version not the mvsc version.

2. Install [Postman](https://www.postman.com/downloads/) 
3. Clone the repository locally and Move to the directory
4. Run the following command to start the server:

    ``` shell
        cargo run --package votingExample --bin votingExample
    ```
5. Open Postman and open a grpc tag
6. Point the tab to ```localhost:8080``` under method click server reflection.
7. Postman will generate an example message for each method for you to try out each type of method in a GRPC service.

## Description

What this to get up to speed with [GRPC](https://www.youtube.com/watch?v=Yw4rkaTc0f8) if you do not know what it is.

This repository is set up to be a full example of all 4 different interaction you can have with GRPC.

Feel free to use this as a base or as a reference.

## Authorization

The Interceptor check_auth is responsible for ensuring the client is authenticated.
Currently, the authorization ensures that a specific Bearer Token, "some-auth-token", is passed to the system. 

This would need to be expanded to check which service(s), should actually have authorization implemented 
and an public endpoint for authentication.

## GRPC Examples

### Unary
#### Status 

The status gives back the number of votes for each candidate that exists currently.

#### Vote

Allows you to vote for an existing candidate or a new one.

### Client Streaming
#### BatchVote
Allows the client to stream votes to the server.

### Server Streaming
#### WatchStream

Streams all votes happening in real time to the client. Returns the new amount for a candidate  

### Full Duplex Streaming
#### VotingStream
Allows the client to stream votes to the server and will echo back the votes that it submitted.

