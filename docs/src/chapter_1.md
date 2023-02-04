
provides a system for remote controlling a pellet dispensing system of which
contains a live camera feed. 




Server CAN communication

- Should always send Servo commands
    while connected send a single message with all servo commands
    while holding "w" send a 1 to indicate commanding that value
    while not holding "w" send a 0 

    rate of sending commands is 1 every 100ms




# User interface documentation


Each servo is tied to two buttons that are opposites, 
so servo 1 is say up/down
and servo 2 is left/right

if a user presses up the servo 1 will move up. 
if a user presses up and then presses down then servo 1 will continue to move up. 

