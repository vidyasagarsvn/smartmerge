using System;
using System.IO;
using System.Text.Json;

namespace SmartMerge.Services
{
    public class AppSettings
    {
        public string? LastLeftPath { get; set; }
        public string? LastRightPath { get; set; }
        public string Theme { get; set; } = "light";
        public string FontFamily { get; set; } = "Consolas";
        public double FontSize { get; set; } = 13;
        public double WindowTop { get; set; } = double.NaN;
        public double WindowLeft { get; set; } = double.NaN;
        public double WindowWidth { get; set; } = 1200;
        public double WindowHeight { get; set; } = 700;
        public string WindowState { get; set; } = "Normal";
    }

    public static class SettingsStore
    {
        private static readonly string SettingsPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            "SmartMerge",
            "settings.json");

        public static AppSettings Load()
        {
            try
            {
                if (!File.Exists(SettingsPath))
                {
                    return new AppSettings();
                }

                var json = File.ReadAllText(SettingsPath);
                var settings = JsonSerializer.Deserialize<AppSettings>(json);
                return settings ?? new AppSettings();
            }
            catch
            {
                return new AppSettings();
            }
        }

        public static void Save(AppSettings settings)
        {
            try
            {
                var dir = Path.GetDirectoryName(SettingsPath);
                if (!string.IsNullOrWhiteSpace(dir))
                {
                    Directory.CreateDirectory(dir);
                }

                var json = JsonSerializer.Serialize(settings, new JsonSerializerOptions
                {
                    WriteIndented = true
                });
                File.WriteAllText(SettingsPath, json);
            }
            catch
            {
                // Ignore settings write failures.
            }
        }
    }
}
