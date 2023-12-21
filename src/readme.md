# Connect $l$

This is a program that lets you play connect 4 in the terminal. The number of columns $w$, height of the columns $h$, and the number of consecutive discs needed to win $l$, can be changed in the menu.

The computer opponent will start by  exploring the tree of possible future moves to a fixed depth $d_0$. If a move takes less than 1 second, then it will increase the calculation depth for the next move. The exploration algorithm is multithreaded and will run one process per non-full column.

The default settings are $l=4$, $w=7$, $h=6$, $d_0=8$. It is far from impossible to defeat but I believe it is advanced enough to demand focus from non-proffesional human players. The difficulty will be hardware dependant since the calculation depth will increase faster and earlier for faster computers.
