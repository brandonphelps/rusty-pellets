# CAN Interface

Bit Value: 0 = Not Pressed, 1 = Pressed

2 Byte Message (0x200) to contain 8 buttons on the keyboard, each taking 1 bit
    
    Byte 1:
    Bit 0 = Up (map to W)
    Bit 1 = Left (map to A)
    Bit 2 = Down (map to S)
    Bit 3 = Right (map to D)
    Bit 4 = Alt_Up (map to Up Arrow)
    Bit 5 = Alt_Left (map to Left Arrow)
    Bit 6 = Alt_Down (map to Down Arrow)
    Bit 7 = Alt_Right (map to Right Arrow)

    Byte 2:
    Bit 0 = Modifier1 (map to Lshift)
    Bit 1 = Modifier2 (map to RShift)
    Bit 2 = Modifier3 (map to left Control)
    Bit 3 = Modifier4 (map to right Control)
    Bit 4 = Generic Input1 (Spacebar)
    Bit 5 = Generic Input2 (Enter)
    Bit 6 = Generic Input3 (map to Q)
    Bit 7 = Generic Input4 (map to E)