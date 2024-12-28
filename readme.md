
# Polling Application

## Backend Documentation

### Overview
The backend is built using **Rust** with the **Actix-web** framework. It provides RESTful APIs for poll creation, management, and real-time updates. The database used is **MongoDB**. Authentication is handled using **Passkey Authentication (WebAuthn/FIDO2)** and **JWT tokens**.

### Features
- User authentication with Passkeys and JWT.
- Poll creation, deletion, and management.
- Real-time updates using Server-Sent Events (SSE).
- Protected routes using JWT custom middleware function.

### Key Components

#### Authentication
- **Routes:**
    - **Authentication**
        - `POST /api/auth/register/start`: Initiates Passkey registration.
        - `POST /api/auth/register/finish`: Completes Passkey registration.
        - `POST /api/auth/login/start`: Initiates Passkey authentication.
        - `POST /api/auth/login/finish`: Completes Passkey authentication.
    
    - **Poll**
        - `GET  /api/`: Retrieves all polls.
        - `POST /api/polls`: Creates a new poll.
        - `GET  /api/polls/[pollId]`: Retrieves poll details.
        - `POST /api/polls/[pollId]/vote`: Casts vote for a poll option.
        - `POST /api/polls/[pollId]/close`: Closes a poll (only for poll creators).
        - `POST /api/polls/[pollId]/reset`: Resets votes for a poll (only for poll creators).

    - **Real Time Updates**
        - `GET /api/socket/create-client`: Creates an SSE client and sends to client for real-time updates.

- **Libraries:**
  - `webauthn-rs` for WebAuthn implementation.
  - `jsonwebtoken` for JWT handling.
  - `actix-web` for Web Server Setup.

#### Database
- **Technology:** MongoDB
- **Structure:**
  - `user` collection for storing user data.
  - `poll` collection for poll details.

### Configuration
- **Environment Variables:**
  - `DATABASE_URL`: MongoDB connection string.
  - `JWT_SECRET`: Secret key for JWT.
  - `DATABASE_NAME`: MongoDB Database name.

### Local Setup

#### Prerequisites
Before setting up the project, make sure you have the following installed:

- **Rust**: Follow the installation guide on [Rust's official website](https://www.rust-lang.org/tools/install).
- **MongoDB**: Install MongoDB locally or use a cloud service like [MongoDB Atlas](https://www.mongodb.com/cloud/atlas).
- **OpenSSL**: Required for Passkey (WebAuthn) authentication. 

#### Setup Steps

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Azeem-0/polling_application_backend.git
   cd polling_application_backend
    ```

2. **Install dependencies**
    ```bash 
      cargo build
    ```
3. **Set up environment variables:**
    ```bash
    DATABASE_URL=<your-mongodb-connection-string>
    JWT_SECRET=<your-secret-key-for-jwt>
    WEBAUTHN_ORIGIN=<your-web-auth-origin-url>
    ```
4. **Run the server:**
  ```bash
    cargo run
  ```
5. **Access the api endpoints as described in the documentation above.**

You can view the documentation of the APIs on ```http://localhost:3001/swagger-ui/.```