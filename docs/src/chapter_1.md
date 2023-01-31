
provides a system for remote controlling a pellet dispensing system of which
contains a live camera feed. 




Server CAN communication

- Should always send Servo commands
    while connected send a single message with all servo commands
    while holding "w" send a 1 to indicate commanding that value
    while not holding "w" send a 0 

    rate of sending commands is 1 every 100ms


