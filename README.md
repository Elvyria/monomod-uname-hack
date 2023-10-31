## Versions
* **tModLoader**: v2023.8.3.4
* **MonoMod.Utils**: 25.0.3

## Building
To build this little thing, you'll need some [Rust](https://www.rust-lang.org/).

```sh
git clone --depth 1 https://github.com/Elvyria/monomod-uname-hack
cd monomod-uname-hack
cargo build --locked --release
```

## Workaround
Hook [libc::uname](https://man7.org/linux/man-pages/man2/uname.2.html) and add `X86_64` or `AMD64` ASCII sequence into the mishandled field(s).

Steam launch options for tModLoader:
```sh
LD_PRELOAD='.../libmonomod_uname_hack.so' %command%
``````

## Reason:
MonoMod is jumping through [utsname](https://man7.org/linux/man-pages/man2/uname.2.html) to access the `machine` field and misses.

`MonoMod.Utils.PlatformDetection.cs`
```cs
var kernelName = GetCString(buffer, out var nullByteOffs).ToUpperInvariant();
buffer = buffer.Slice(nullByteOffs);

for (var i = 0; i < 4; i++) { // we want to jump to string 4, but we've already skipped the text of the first
    if (i != 0) {
        // skip a string
        nullByteOffs = buffer.IndexOf((byte)0);
        buffer = buffer.Slice(nullByteOffs);
    }
    // then advance to the next one
    var j = 0;
    for (; j < buffer.Length && buffer[j] == 0; j++) { }
    buffer = buffer.Slice(j);
}

// and here we find the machine field
var machineName = GetCString(buffer, out _).ToUpperInvariant();
```
`machineName` is then searched for `X86_64` and `AMD64` substrings and if fails - the whole thing just refuses to work, because `x86_64` is the only accepted architecture down the line.

## Issue 
#### LinuxSystem.cs:line 46
This is a manually thrown exception that happens if the machine architecture doesn't match x86_64.
```cs
System.NotImplementedException: The method or operation is not implemented.
   at MonoMod.Core.Platforms.Systems.LinuxSystem..ctor() in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.Core\Platforms\Systems\LinuxSystem.cs:line 46
   at MonoMod.Core.Platforms.PlatformTriple.CreateCurrentSystem() in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.Core\Platforms\PlatformTriple.cs:line 83
   at MonoMod.Core.Platforms.PlatformTriple.CreateCurrent() in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.Core\Platforms\PlatformTriple.cs:line 116
   at MonoMod.Utils.Helpers.InitializeValueWithLock[T,TParam](T& location, Object lock, IntPtr init, TParam obj) in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.Utils\Helpers.cs:line 187
   at MonoMod.Core.Platforms.PlatformTriple.get_Current() in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.Core\Platforms\PlatformTriple.cs:line 113
   at MonoMod.Core.DetourFactory.CreateDefaultFactory() in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.Core\IDetourFactory.cs:line 80
   at MonoMod.Utils.Helpers.InitializeValue[T,TParam](T& location, IntPtr init, TParam obj) in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.Utils\Helpers.cs:line 178
   at MonoMod.RuntimeDetour.DetourContext.GetDefaultFactory() in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.RuntimeDetour\DetourContext.cs:line 104
   at MonoMod.RuntimeDetour.Hook..ctor(MethodBase source, Delegate target, DetourConfig config, Boolean applyByDefault) in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.RuntimeDetour\Hook.cs:line 410
   at MonoMod.RuntimeDetour.Hook..ctor(MethodBase source, Delegate target, DetourConfig config) in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.RuntimeDetour\Hook.cs:line 378
   at MonoMod.RuntimeDetour.Hook..ctor(MethodBase source, Delegate target) in Z:\Users\aaron\Source\Repos\MonoModReorg\MonoMod\src\MonoMod.RuntimeDetour\Hook.cs:line 320
   at Terraria.ModLoader.Engine.LoggingHooks.PrettifyStackTraceSources() in D:\a\tModLoader\tModLoader\src\tModLoader\Terraria\ModLoader\Engine\LoggingHooks.cs:line 73
   at Terraria.ModLoader.Engine.LoggingHooks.Init() in D:\a\tModLoader\tModLoader\src\tModLoader\Terraria\ModLoader\Engine\LoggingHooks.cs:line 17
   at Terraria.ModLoader.Logging.LogStartup(Boolean dedServ) in D:\a\tModLoader\tModLoader\src\tModLoader\Terraria\ModLoader\Logging.cs:line 93
   at Terraria.Program.StartupSequenceTml(Boolean isServer) in D:\a\tModLoader\tModLoader\src\tModLoader\Terraria\Program.TML.cs:line 284
```
