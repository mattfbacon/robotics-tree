---
name = "Programming the Drivetrain with VEXCode Pro v5"
description = "In this document, we upload a basic program to a pre-built drivetrain to make the robot move with driver control! If you haven't built the drivetrain already, refer to 'Building a Drivetrain'."
---

## Step 1

Gather materials:

- a computer with [VEXCode **_Pro_** v5](https://www.vexrobotics.com/vexcode/install/v5) installed
- an upload cable
  - not all microUSB cables can work as an upload cable; the cable must be "data carrying".
- a built drivetrain

## Step 2

Click "File" on the top navbar, then hit "New".



![VEXCode Pro home screen with "New" highlighted](./2-2-1.png)

## Step 3

Click the motor icon on the top right; then click "Add a device".



!["Add a device" button under "Robot Configuration"](./2-3-1.png)

\
Add a "Controller", then click "DONE".



![VEX configuration device list with "Controller" selected](./2-3-2.png)
!["Controller1" configuration panel with "DONE" highlighted](./2-3-3.png)



## Step 4

Add a motor by clicking "Add a Device" again, then selecting "MOTOR".



![VEX configuration device list with "MOTOR" selected](./2-4-1.png)

\
Select a port (doesn't matter for now), then name the motor "left_motor" by clicking the text field near the top. Finally, click "DONE". You should see the port number of left_motor near its name.



![motor configuration screen; renamed to "left_motor"](./2-4-2.png)
![left_motor connected to port 3, indicated with "3" icon](./2-4-3.png)

\
Add another motor named "right_motor" with the same process outlined above. You should have two motors and a Controller in your "Robot Configuration" list.



![Controller1, left_motor, and right_motor listed in "Robot Configuration"](./2-4-4.png)

## Step 5

Locate the `main` function in the code. The function starts with `int main() {` and ends with a `}` character. Place your cursor inside the `main` function, after the `vexcodeInit();` command.



![inside main() {, after vexcodeInit()](./2-5-1.png)

Paste the following code inside the `main` function, at your cursor:

```cpp
while (true) {
    left_motor.spin(forward, Controller1.Axis3.position() * drivetrain_dampening, percent); //spin drivetrain
    right_motor.spin(forward, Controller1.Axis2.position() * drivetrain_dampening, percent);
    wait(100, msec);
  }
```

You should get a red underline under `drivetrain_dampening`; when hovering, an error pops up that `[clang] Use of undeclared identifier 'drivetrain_dampening'`. This error basically says that we haven't defined the `drivetrain_dampening` variable yet (if you don't know what variables are, don't worry about it for now).



![drivetrain_dampening isn't defined](./2-5-2.png)

Define the `drivetrain_dampening` outside of the `main` function with this code:

```cpp
float drivetrain_dampening = 1;
```

Your full `main.cpp` file should now look like this:

```cpp
#include "vex.h"

using namespace vex;

float drivetrain_dampening = 1;

int main() {
  // Initializing Robot Configuration. DO NOT REMOVE!
  vexcodeInit();
  while (true) {
    left_motor.spin(forward, Controller1.Axis3.position() * drivetrain_dampening, percent); //spin drivetrain
    right_motor.spin(forward, Controller1.Axis2.position() * drivetrain_dampening, percent);
    wait(100, msec);
  }
}
```

The bit inside the `main` function tells the robot motors to spin according to your `Controller1` axis values; a Controller "axis" is just a way to move the joystick (e.g. up and down on the left joystick, left to right on the right joystick, up and down on the right joystick, etc.)

## Step 6

Now the fun bit -- uploading your code and testing!

- **Plug in the motors to the ports you configured them to.** For instance, if you configured the `left_motor` to be on port 3, then connect the left motor to port 3.
- Turn on your CORTEX and plug in the upload cable to both your computer and the CORTEX.
- The buttons on the top right of the screen should change to show that your robot was plugged in. Find the Upload/Download button and click it.
- After the program is uploaded, it should show up in the list of programs on the CORTEX.

## Step 7

Run the drivetrain program, and you should be able to control the drivetrain with your joysticks!
