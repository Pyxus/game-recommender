# Game Recommendation Tool

<p align="center">
    <img src="demo.gif" alt="Interpreter demo">
</p>

Game recommendation tool. The user provides games they enjoy as an input and the server generates a list of recommended games using a content based filtering algorithm.

## Introduction

This tool is a project developed primarily to practice my skill in Rust. Users can input games they enjoy and the recommender will generate a list of recommendations based on the input games.

### Technologies

This tool is built using the following technologies:

- Backend (Rust):
  - [Rocket](https://rocket.rs/) - Used to create RESTful API which facilitates communication between the frontend and backend.
  - [Nalgebra](https://nalgebra.org/) - Used in my implementation of the [content based filtering algorithm](https://developers.google.com/machine-learning/recommendation/content-based/basics).
- Frontend (React)
- External Services:
  - [IGDB](https://www.igdb.com/) - Used to source relevant game information

### Installation

The recommender is not currently hosted anywhere and since it was just for practice I didn't intend for anyone to use it. However, if you'd like to play with it below are some simple steps on how you could go about that.

1. Clone the repository

```shell
git clone https://github.com/Pyxus/game-recommender.git
cd game-recommender
```

2. Setup the server:
   - Install Rust and Cargo. Refer to the [Rust Installation Guide](https://www.rust-lang.org/tools/install).
   - Obtain an API key from [IGDB](https://www.igdb.com/api)
   - Create a `.env` file in the `server` directory.
   ```makefile
   TWITCH_CLIENT_ID=YOUR_ID_HERE
   TWITCH_CLIENT_SECRET=YOUR_SECRET_HERE
   ```
   - Build and run the server
   ```shell
   cd server
   cargo run
   ```
3. Setup the frontend:
   - Install NodeJs. Refer to [Node.js Downloads](https://nodejs.org/en/download)
   - Install dependencies:
   ```shell
   cd client
   npm install
   ```
   - Start frontend
   ```shell
   npm run dev
   ```
   - The project should now be live on your localhost.
