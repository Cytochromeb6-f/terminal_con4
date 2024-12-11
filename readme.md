# Connect $l$

This is a program that lets you play connect 4 in the terminal. The number of columns: $w$, height of the columns: $h$, and the number of consecutive discs needed to win: $l$, can be changed in the menu.

## AI

The computer opponent will start by exploring the tree of possible future moves to a fixed depth $d_0$. If a move takes less than 1 second, then it will increase the calculation depth for the next move. This means that the difficulty has a slight hardware dependence since the calculation depth will increase faster and earlier for faster computers.

The computer opponent will use a different algorithm depending on if the column height $h$ is even or not.

If $h$ is odd then it uses a BFS search and picks the branch where it has the highest number of ways it can win compared to the number of ways it can lose. This exploration algorithm is multithreaded and will run one process per non-full column.

When $h$ is even then every horizontal and diagonal threats have an intrinsic parity. In this case the computer opponent will use a minmax search with a heuristic that prioritizes low threats with the prefered parity. This algorithm is enhanced with $\alpha\text{--}\beta$ pruning and repeated states caching. This makes it much faster than the other algorithm.

## Difficulty

On grids with odd height the strength of the computer opponent is comparable to non-proffesional human players.

It is much stronger on grids with even height. I have been unable to defeat the default settings:
$$
    l=4,\; w=7,\; h=6,\; d_0=10.
$$
With $d_0=11$ it easily beats most bots I have found online. It is not perfect but with $d_0=12$ it can sometimes draw against a perfect opponent on a standard grid.
