# Software Challenge 2023 Rust Client

Client by Team Gym Kenzingen K1, [winner of the Software Challenge Germany 2023](https://contest.software-challenge.de/seasons/2023/contests/142/finale)

This client is based on the Monte Carlo Tree Search algorithm but has been refined and specifically optimised for the competition's selected board game. Unlike the traditional MCTS algorithm, it uses a customized heuristic function to evaluate the territory of each player by a given game state. Additionally, it uses caching to store the game tree containing the results of the previous explorations.