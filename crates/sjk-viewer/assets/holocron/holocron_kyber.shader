// The kyber crystals (SJK; see README.md): each crystal's facets, lit, glowing of
// themselves, their lit parts breathing in the glow and a sheen stripe sliding
// across, brighter the rarer; then each crystal's light (lamps, motes).
// Made by scripts/holocron_assets.py from its TIERS table: edit that, not this.
models/sjk/holocron_kyber_uncommon
{
	q3map_nolightmap
	{
		map models/sjk/holocron_kyber_uncommon
		rgbGen lightingDiffuse
	}
	{
		map models/sjk/holocron_kyber_uncommon
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.35 0 0 0
	}
	{
		map models/sjk/holocron_kyber_uncommon_glow
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.4 0.1 0 0.3
		glow
	}
	{
		map models/sjk/holocron_sheen
		blendFunc GL_ONE GL_ONE
		tcMod scale 0.5 0.5
		tcMod scroll 0.06 0.06
		rgbGen wave sin 0.2 0.15 0 0.25
	}
}
models/sjk/holocron_kyber_rare
{
	q3map_nolightmap
	{
		map models/sjk/holocron_kyber_rare
		rgbGen lightingDiffuse
	}
	{
		map models/sjk/holocron_kyber_rare
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.4 0 0 0
	}
	{
		map models/sjk/holocron_kyber_rare_glow
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.5 0.15 0 0.35
		glow
	}
	{
		map models/sjk/holocron_sheen
		blendFunc GL_ONE GL_ONE
		tcMod scale 0.5 0.5
		tcMod scroll 0.06 0.06
		rgbGen wave sin 0.3 0.15 0 0.25
	}
}
models/sjk/holocron_kyber_legendary
{
	q3map_nolightmap
	{
		map models/sjk/holocron_kyber_legendary
		rgbGen lightingDiffuse
	}
	{
		map models/sjk/holocron_kyber_legendary
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.45 0 0 0
	}
	{
		map models/sjk/holocron_kyber_legendary_glow
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.6 0.2 0 0.4
		glow
	}
	{
		map models/sjk/holocron_sheen
		blendFunc GL_ONE GL_ONE
		tcMod scale 0.5 0.5
		tcMod scroll 0.06 0.06
		rgbGen wave sin 0.4 0.15 0 0.25
	}
}
models/sjk/holocron_kyber_mythical
{
	q3map_nolightmap
	{
		map models/sjk/holocron_kyber_mythical
		rgbGen lightingDiffuse
	}
	{
		map models/sjk/holocron_kyber_mythical
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.45 0.2 0.25 0.35
	}
	{
		map models/sjk/holocron_kyber_mythical_glow
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.85 0.35 0 0.55
		glow
	}
	{
		map models/sjk/holocron_sheen
		blendFunc GL_ONE GL_ONE
		tcMod scale 0.5 0.5
		tcMod scroll 0.06 0.06
		rgbGen wave sin 0.5 0.15 0 0.25
	}
}
models/sjk/holocron_kyber_uncommon_light
{
	q3map_nolightmap
	{
		map models/sjk/holocron_kyber_uncommon_light
		rgbGen identity
	}
	{
		map models/sjk/holocron_kyber_uncommon_light
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.4 0.1 0 0.3
		glow
	}
}
models/sjk/holocron_kyber_rare_light
{
	q3map_nolightmap
	{
		map models/sjk/holocron_kyber_rare_light
		rgbGen identity
	}
	{
		map models/sjk/holocron_kyber_rare_light
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.5 0.15 0 0.35
		glow
	}
}
models/sjk/holocron_kyber_legendary_light
{
	q3map_nolightmap
	{
		map models/sjk/holocron_kyber_legendary_light
		rgbGen identity
	}
	{
		map models/sjk/holocron_kyber_legendary_light
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.6 0.2 0 0.4
		glow
	}
}
models/sjk/holocron_kyber_mythical_light
{
	q3map_nolightmap
	{
		map models/sjk/holocron_kyber_mythical_light
		rgbGen identity
	}
	{
		map models/sjk/holocron_kyber_mythical_light
		blendFunc GL_ONE GL_ONE
		rgbGen wave sin 0.85 0.35 0 0.55
		glow
	}
}
