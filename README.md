# Javaver

Sets the Java version automatically using command line.
You can add (both manually and automatically) to the list of SDKs with custom names, as well as remove them.
Use the names to select the SDK you want, which will automatically update the environment variables for you.

Setting the Java version (using "sel" attribute) requires **administrator privileges**.

## Installation

Download the latest version in Releases in this repository. You can now use the javaver.exe in terminal.

### Adding to path

Move the executable to the desired directory. Afterwards, add that directory to the path manually, or using shell:

`setx /M path "%path%;C:\your\path\here\"`

**WARNING: using the provided shell script will truncate the input data to 1024 characters. This is most likely insufficient. Make sure you know what you are doing.**

**Be careful and responsible.**

## Usage

Use "javaver --help" for help in using the utility.
