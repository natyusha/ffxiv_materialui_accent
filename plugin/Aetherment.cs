global using Dalamud.Logging;

using System;
using System.IO;
using System.Diagnostics;
using System.Threading.Tasks;
using System.Collections.Generic;
using Newtonsoft.Json;

using ImGuiScene;
using Dalamud.IoC;
using Dalamud.Game;
using Dalamud.Plugin;
using Dalamud.Interface;
using Dalamud.Game.Command;
using Dalamud.Game.Gui;

using Lumina;

using Aetherment.GUI;
using Aetherment.Util;

namespace Aetherment {
	[Serializable]
	public class Config {
		public int Version = 0;
		
		// public List<string> InstalledMods = new();
		
		// public bool AutoUpdate = true;
		public bool LinkOptions = true;
		
		public bool AdvancedMode = false;
		public bool ForceColor4 = false;
		public bool LocalMods = false;
		public string LocalModsPath = "";
		public List<GitHub.RepoInfo> Repos = new();
		
		public bool DevMode = false;
		
		public string ExplorerMod = "";
		public string ExplorerExportPath = ".";
		public Dictionary<string, string> ExplorerExportExt = new();
	}
	
	public class Aetherment : IDalamudPlugin {
		public string Name => "Aetherment";
		private const string command = "/aetherment";
		private const string commandAlt = "/materialui";
		private const string commandFinder = "/texfinder";
		
		[PluginService][RequiredVersion("1.0")] public static DalamudPluginInterface Interface  {get; private set;} = null!;
		[PluginService][RequiredVersion("1.0")] public static CommandManager         Commands   {get; private set;} = null!;
		[PluginService][RequiredVersion("1.0")] public static TitleScreenMenu        TitleMenu  {get; private set;} = null!;
		
		internal static GameData GameData;
		
		internal static Config Config;
		internal static UI Ui;
		internal static TextureFinder TextureFinder;
		
		internal static Dictionary<string, TextureWrap> Textures = new();
		internal static List<Mod> InstalledMods = new();
		
		public Aetherment() {
			Installer.Initialize();
			foreach(var file in new DirectoryInfo(Interface.AssemblyLocation.DirectoryName + "/assets/icons").EnumerateFiles())
				Textures[file.Name] = Interface.UiBuilder.LoadImage(file.FullName);
			
			GameData = new GameData(Path.GetDirectoryName(Process.GetCurrentProcess().MainModule.FileName) + "/sqpack", new LuminaOptions());
			
			var path = $"{Interface.ConfigDirectory.FullName}/config.json";
			Config = File.Exists(path) ? JsonConvert.DeserializeObject<Config>(File.ReadAllText(path)) : new Config();
			Config.Repos.Insert(0, new GitHub.RepoInfo("Sevii77", "ffxiv_materialui_accent", "v2"));
			Ui = new();
			TextureFinder = new();
			
			foreach(var modid in PenumbraApi.GetMods())
				AddLocalMod(modid);
			
			Task.Run(async() => {
				foreach(var m in InstalledMods)
					if(m.AutoUpdate) {
						var mod = await Mod.GetMod(m.Repo, m.ID);
						if(mod != null)
							Installer.DownloadMod(mod);
					}
			});
			
			Commands.AddHandler(command, new CommandInfo(OnCommand) {
				HelpMessage = "Open Aetherment menu"
			});
			Commands.AddHandler(commandAlt, new CommandInfo(OnCommand) {
				HelpMessage = "Alternative for /aetherment"
			});
			Commands.AddHandler(commandFinder, new CommandInfo(OnCommand) {
				HelpMessage = "Open Texture Finder, used to find any ui texture"
			});
			
			// Task.Run(async() => {
			// 	PathsDB.Fetch();
			// });
		}
		
		public void Dispose() {
			Installer.Dispose();
			Ui.Dispose();
			TextureFinder.Dispose();
			
			Commands.RemoveHandler(command);
			Commands.RemoveHandler(commandAlt);
			Commands.RemoveHandler(commandFinder);
			
			foreach(var texture in Textures.Values)
				texture.Dispose();
		}
		
		public static void AddInstalledMod(Mod mod) {
			// if(!Config.InstalledMods.Contains(id))
			// 	Config.InstalledMods.Add(id);
			
			// Aetherment.Ui.AddLocalMod(id);
			// SaveConfig();
			
			if(InstalledMods.Exists(x => x.ID == mod.ID))
				return;
			
			lock(InstalledMods)
				InstalledMods.Add(mod);
		}
		
		public static void AddLocalMod(string id) {
			if(InstalledMods.Exists(x => x.ID == id))
				return;
			
			try {
				var mod = Mod.GetModLocal(id);
				if(mod != null)
					lock(InstalledMods)
						InstalledMods.Add(mod);
			} catch(Exception e) {
				PluginLog.Error(e, $"Failed adding local mod {id}");
				// Task.Run(async() => {
				// 	foreach(var repo in Config.Repos) {
				// 		var mod = await Mod.GetMod(repo, id);
				// 		if(mod != null)
				// 			Installer.DownloadMod(mod);
				// 	}
				// });
			}
		}
		
		public static void DeleteInstalledMod(string id) {
			// Config.InstalledMods.Remove(id);
			InstalledMods.RemoveAll(x => x.ID == id);
			// Aetherment.Ui.DeleteLocalMod(id);
		}
		
		public static void SaveConfig() {
			Config.Repos.RemoveAt(0); // lol idc anymore for now
			File.WriteAllText($"{Aetherment.Interface.ConfigDirectory.FullName}/config.json", JsonConvert.SerializeObject(Config));
			Config.Repos.Insert(0, new GitHub.RepoInfo("Sevii77", "ffxiv_materialui_accent", "v2"));
		}
		
		private void OnCommand(string cmd, string args) {
			if(cmd == command || cmd == commandAlt)
				Ui.Show();
			else if(cmd == commandFinder)
				TextureFinder.Show();
		}
	}
}