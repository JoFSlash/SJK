/*
 * The hints that make a laptop with two graphics cards run sjk.exe on its fast one
 * (built only for Windows, by build.rs). NVIDIA Optimus and AMD switchable graphics
 * read these exported values when the program starts: without them the driver may
 * start the game on the integrated card, where it closed at once on some laptops
 * (#sjk player chat, 28-29/10/2026: SimpleSmokie's laptop ran once Nib set sjk.exe to
 * the NVIDIA card in the NVIDIA Control Panel). The values are the drivers' documented
 * ones; C, because the client's Rust forbids the unmangled statics they need.
 */
__declspec(dllexport) unsigned long NvOptimusEnablement = 0x00000001;
__declspec(dllexport) int AmdPowerXpressRequestHighPerformance = 1;
