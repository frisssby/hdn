# **hdn**

Hash delivery network is a tcp server that supports storing hashes in a remote storage and loading them from it. Supports running on several nodes with data synchronization.

## Config

Server requires a configuration file which is supposed to have the following structure:

```json
{
    "user": "Elina Safarova",
    "nodes": [
        "127.0.0.1",
        "127.0.0.2",
        "127.0.0.3"
    ],
    "id": 0, // index of the server in the nodes list 
    "client_port": 43500,
    "peer_port": 42300
}
```

Before launching a server, put the path to the configuration file to the **HDN_CONFIG**
environment variable.

## Launching

```bash
git clone git@github.com:frisssby/hdn.git
cd hdn
cargo run
```

## Protocol

Communication with the server occurs through **json** messages.

After the successful connection, the client receives a greeting message:

```json
  {
    "student_name" : "Elina Safarova",
  }
```

There are two types of valid requests:

+ **Store** a *hash* under a *key*

    ```json
    {
      "request_type": "store",
      "key": "some_key",
      "hash": "0b672dd94fd3da6a8d404b66ee3f0c8",
    }
    ```

    In case the request went successful, the server responds the client with:

     ```json
    {
      "response_status": "success",
    }
    ```

+ **Load** the previously saved hash by a key

    ```json
    {
      "request_type": "load",
      "key": "some_key",
    }
    ```

    In case the request went successful, the server responds the client with:

    ```json
    {
      "response_status": "success",
      "requested_key": "some_key",
      "requested_hash": "0b672dd94fd3da6a8d404b66ee3f0c83",
    }
    ```

    If the key is not in the storage, the server will send:

    ```json
    {
      "response_status": "key not found",
    }
    ```

+ If the server gets an invalid request, it will reply with a message:

    ```json
    {
      "response_status": "invalid request",
    }
    ```
