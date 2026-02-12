using System;
using System.IO;
using System.Windows;
using System.Windows.Media;
using Microsoft.Win32;

namespace SmartMerge.Views.Dialogs
{
    public partial class OpenFilesDialog : Window
    {
        public string LeftPath { get; private set; } = string.Empty;
        public string RightPath { get; private set; } = string.Empty;

        public OpenFilesDialog(string? initialLeft, string? initialRight)
        {
            InitializeComponent();
            LeftPath = initialLeft ?? string.Empty;
            RightPath = initialRight ?? string.Empty;
            LeftPathBox.Text = LeftPath;
            RightPathBox.Text = RightPath;
            UpdateStatus();
        }

        private void BrowseLeft(object sender, RoutedEventArgs e)
        {
            var dialog = new OpenFileDialog
            {
                Title = "Select Left File",
                Filter = "All Files|*.*"
            };

            if (dialog.ShowDialog(this) == true)
            {
                LeftPath = dialog.FileName;
                LeftPathBox.Text = LeftPath;
                UpdateStatus();
            }
        }

        private void BrowseRight(object sender, RoutedEventArgs e)
        {
            var dialog = new OpenFileDialog
            {
                Title = "Select Right File",
                Filter = "All Files|*.*"
            };

            if (dialog.ShowDialog(this) == true)
            {
                RightPath = dialog.FileName;
                RightPathBox.Text = RightPath;
                UpdateStatus();
            }
        }

        private void UpdateStatus()
        {
            LeftStatus.Text = BuildStatus(LeftPath, out var leftBrush);
            RightStatus.Text = BuildStatus(RightPath, out var rightBrush);
            LeftStatus.Foreground = leftBrush;
            RightStatus.Foreground = rightBrush;
            CompareButton.IsEnabled = File.Exists(LeftPath) && File.Exists(RightPath);
        }

        private static string BuildStatus(string path, out Brush brush)
        {
            if (string.IsNullOrWhiteSpace(path))
            {
                brush = new SolidColorBrush(Color.FromRgb(107, 114, 128));
                return "Select a file to compare";
            }

            if (!File.Exists(path))
            {
                brush = new SolidColorBrush(Color.FromRgb(244, 67, 54));
                return "File not found";
            }

            var info = new FileInfo(path);
            brush = new SolidColorBrush(Color.FromRgb(76, 175, 80));
            return $"File exists - {FormatSize(info.Length)}";
        }

        private static string FormatSize(long size)
        {
            var units = new[] { "B", "KB", "MB", "GB", "TB" };
            var value = (double)size;
            var unitIndex = 0;
            while (value >= 1024 && unitIndex < units.Length - 1)
            {
                value /= 1024;
                unitIndex++;
            }

            return $"{value:0.0} {units[unitIndex]}";
        }

        private void Compare(object sender, RoutedEventArgs e)
        {
            DialogResult = true;
        }

        private void Cancel(object sender, RoutedEventArgs e)
        {
            DialogResult = false;
        }
    }
}
