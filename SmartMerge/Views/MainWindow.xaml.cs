using System;
using System.ComponentModel;
using System.Linq;
using System.Windows;
using SmartMerge.Services;
using SmartMerge.ViewModels;

namespace SmartMerge.Views
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        private readonly Uri _lightThemeUri = new("Themes/Light.xaml", UriKind.Relative);
        private readonly Uri _darkThemeUri = new("Themes/Dark.xaml", UriKind.Relative);

        public MainWindow()
        {
            InitializeComponent();
            if (DataContext == null)
            {
                DataContext = new MainWindowViewModel();
            }

            Loaded += OnLoaded;
            Closing += OnClosing;
        }

        private void OnLoaded(object sender, RoutedEventArgs e)
        {
            if (DataContext is not MainWindowViewModel vm)
            {
                return;
            }

            ApplyTheme(vm.Theme);
            RestoreWindowState(vm.Settings);

            vm.PropertyChanged += OnViewModelPropertyChanged;

            if (FolderCompareViewControl != null)
            {
                FolderCompareViewControl.FolderSelected += (_, item) => vm.NavigateIntoFolder(item);
                FolderCompareViewControl.FileSelected += (_, item) => vm.CompareFileFromFolder(item);
            }
        }

        private void OnViewModelPropertyChanged(object? sender, PropertyChangedEventArgs e)
        {
            if (sender is not MainWindowViewModel vm)
            {
                return;
            }

            if (e.PropertyName == nameof(MainWindowViewModel.Theme))
            {
                ApplyTheme(vm.Theme);
            }
        }

        private void RestoreWindowState(AppSettings settings)
        {
            if (!double.IsNaN(settings.WindowWidth) && settings.WindowWidth > 0)
            {
                Width = settings.WindowWidth;
            }

            if (!double.IsNaN(settings.WindowHeight) && settings.WindowHeight > 0)
            {
                Height = settings.WindowHeight;
            }

            if (!double.IsNaN(settings.WindowLeft) && !double.IsNaN(settings.WindowTop))
            {
                Left = settings.WindowLeft;
                Top = settings.WindowTop;
            }

            if (Enum.TryParse(settings.WindowState, out WindowState state))
            {
                WindowState = state;
            }
        }

        private void ApplyTheme(string theme)
        {
            var dictionary = new ResourceDictionary
            {
                Source = theme == "dark" ? _darkThemeUri : _lightThemeUri
            };

            var merged = Application.Current.Resources.MergedDictionaries;
            var existing = merged.FirstOrDefault(d => d.Source != null && d.Source.OriginalString.Contains("Themes/", StringComparison.OrdinalIgnoreCase));
            if (existing != null)
            {
                var index = merged.IndexOf(existing);
                merged[index] = dictionary;
            }
            else
            {
                merged.Add(dictionary);
            }
        }

        private void OnClosing(object? sender, CancelEventArgs e)
        {
            if (DataContext is not MainWindowViewModel vm)
            {
                return;
            }

            if (!vm.ConfirmProceedWithUnsavedChanges("Quit"))
            {
                e.Cancel = true;
                return;
            }

            vm.Settings.WindowState = WindowState.ToString();
            if (WindowState == WindowState.Normal)
            {
                vm.Settings.WindowWidth = Width;
                vm.Settings.WindowHeight = Height;
                vm.Settings.WindowLeft = Left;
                vm.Settings.WindowTop = Top;
            }
            else
            {
                vm.Settings.WindowWidth = RestoreBounds.Width;
                vm.Settings.WindowHeight = RestoreBounds.Height;
                vm.Settings.WindowLeft = RestoreBounds.Left;
                vm.Settings.WindowTop = RestoreBounds.Top;
            }

            SettingsStore.Save(vm.Settings);
        }
    }
}
