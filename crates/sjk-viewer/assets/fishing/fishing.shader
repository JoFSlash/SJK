// The Fishing toy's rod, bucket, float, line and fish (SJK; see README.md): flat
// colours from one small atlas, lit by the world's light as a player model is, seen
// from both sides so the open bucket and the thin line never show holes.
models/sjk/fishing
{
	cull none
	{
		map models/sjk/fishing
		rgbGen lightingDiffuse
	}
}

// The pond's water: ripples that drift and sway, lit as the rest.
models/sjk/fishing_pond_water
{
	cull none
	{
		map models/sjk/fishing_pond
		rgbGen lightingDiffuse
		tcMod turb 0 0.02 0 0.15
		tcMod scroll 0.03 0.015
	}
}
