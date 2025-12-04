mod greet;

// Collect the commands of the application
//
// This macro collects all the command functions defined in this module and its sub-modules,
// and registers them with the Clawless runtime so they can be invoked from the command line.
clawless::commands!();
