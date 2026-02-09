# Software Architecture

This document describes the software architecture of the project. It provides an overview of the main components, their interactions, and the design principles followed during development.

The large parts are a server and a client. The server is responsible for handling requests, processing data, and managing the database. The client provides a user interface for interacting with the server and displaying data to the user.

Even if the game ends up as single-player, the client-server architecture allows for better separation of concerns and easier maintenance. The server can be developed and tested independently of the client, and vice versa. Additionally, this architecture allows for future expansion, such as adding multiplayer functionality or integrating with other services.
