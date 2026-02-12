using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.IO;
using System.Runtime.CompilerServices;
using System.Windows;
using System.Windows.Input;
using SmartMerge.Models;
using SmartMerge.Services;
using SmartMerge.Utils;
using SmartMerge.Views.Dialogs;

namespace SmartMerge.ViewModels
{
    public enum AppView
    {
        FileCompare,
        FolderCompare
    }

    public class MainWindowViewModel : INotifyPropertyChanged
    {
        private readonly AppSettings _settings;
        private readonly Stack<(DirectoryInfo Left, DirectoryInfo Right)> _navigationStack = new();
        private AppView _currentView = AppView.FileCompare;
        private bool _canBackToFolder;
        private string _statusMessage = "Ready";
        private SearchDialog? _searchDialog;

        public MainWindowViewModel()
        {
            _settings = SettingsStore.Load();
            FileCompare = new FileCompareViewModel();
            FolderCompare = new FolderCompareViewModel();

            Theme = "light";
            FileCompare.DiffFontFamily = new System.Windows.Media.FontFamily(_settings.FontFamily);
            FileCompare.DiffFontSize = _settings.FontSize > 0 ? _settings.FontSize : 13;
            if (!string.IsNullOrWhiteSpace(_settings.LastLeftPath))
            {
                LastLeftPath = _settings.LastLeftPath;
            }

            if (!string.IsNullOrWhiteSpace(_settings.LastRightPath))
            {
                LastRightPath = _settings.LastRightPath;
            }

            FileCompare.PropertyChanged += (_, args) =>
            {
                if (args.PropertyName == nameof(FileCompareViewModel.StatusInfo))
                {
                    StatusMessage = FileCompare.StatusInfo;
                }
                if (args.PropertyName == nameof(FileCompareViewModel.HasUndo))
                {
                    CommandManager.InvalidateRequerySuggested();
                }
                if (args.PropertyName == nameof(FileCompareViewModel.HasRedo))
                {
                    CommandManager.InvalidateRequerySuggested();
                }
                if (args.PropertyName == nameof(FileCompareViewModel.SelectedDiffEngine))
                {
                    OnPropertyChanged(nameof(IsMyersEngine));
                    OnPropertyChanged(nameof(IsSmartEngine));
                }
            };

            OpenFilesCommand = new RelayCommand(_ => OpenFiles());
            OpenFoldersCommand = new RelayCommand(_ => OpenFolders());
            BackToFolderCommand = new RelayCommand(_ => BackToFolder(), _ => CanBackToFolder);
            CopyLeftToRightCommand = new RelayCommand(_ => FileCompare.CopyToRight(), _ => CurrentView == AppView.FileCompare);
            CopyRightToLeftCommand = new RelayCommand(_ => FileCompare.CopyToLeft(), _ => CurrentView == AppView.FileCompare);
            CopyAllLeftToRightCommand = new RelayCommand(_ => FileCompare.CopyAllToRight(), _ => CurrentView == AppView.FileCompare);
            CopyAllRightToLeftCommand = new RelayCommand(_ => FileCompare.CopyAllToLeft(), _ => CurrentView == AppView.FileCompare);
            PrevCommand = new RelayCommand(_ => FileCompare.PreviousChange(), _ => CurrentView == AppView.FileCompare);
            NextCommand = new RelayCommand(_ => FileCompare.NextChange(), _ => CurrentView == AppView.FileCompare);
            UndoCommand = new RelayCommand(_ => FileCompare.Undo(), _ => FileCompare.HasUndo);
            RedoCommand = new RelayCommand(_ => FileCompare.Redo(), _ => FileCompare.HasRedo);
            SaveCommand = new RelayCommand(_ => SaveModifiedFiles());
            SaveAllCommand = new RelayCommand(_ => SaveModifiedFiles());
            FindCommand = new RelayCommand(_ => OpenFindDialog(), _ => CurrentView == AppView.FileCompare);
            GoToLineCommand = new RelayCommand(_ => OpenGoToLineDialog(), _ => CurrentView == AppView.FileCompare);
            FontCommand = new RelayCommand(_ => OpenFontDialog());
            ExitCommand = new RelayCommand(_ => Exit());
            AboutCommand = new RelayCommand(_ => About());

            UseMyersCommand = new RelayCommand(_ => FileCompare.SelectedDiffEngine = DiffEngineType.Myers);
            UseSmartCommand = new RelayCommand(_ => FileCompare.SelectedDiffEngine = DiffEngineType.Smart);
        }

        public FileCompareViewModel FileCompare { get; }
        public FolderCompareViewModel FolderCompare { get; }

        public ICommand OpenFilesCommand { get; }
        public ICommand OpenFoldersCommand { get; }
        public ICommand BackToFolderCommand { get; }
        public ICommand CopyLeftToRightCommand { get; }
        public ICommand CopyRightToLeftCommand { get; }
        public ICommand CopyAllLeftToRightCommand { get; }
        public ICommand CopyAllRightToLeftCommand { get; }
        public ICommand PrevCommand { get; }
        public ICommand NextCommand { get; }
        public ICommand UndoCommand { get; }
        public ICommand RedoCommand { get; }
        public ICommand SaveCommand { get; }
        public ICommand SaveAllCommand { get; }
        public ICommand FindCommand { get; }
        public ICommand GoToLineCommand { get; }
        public ICommand FontCommand { get; }
        public ICommand ExitCommand { get; }
        public ICommand AboutCommand { get; }
        public ICommand UseMyersCommand { get; }
        public ICommand UseSmartCommand { get; }

        public AppView CurrentView
        {
            get => _currentView;
            set
            {
                if (_currentView == value)
                {
                    return;
                }

                _currentView = value;
                OnPropertyChanged();
                OnPropertyChanged(nameof(IsFileViewActive));
                OnPropertyChanged(nameof(IsFolderViewActive));
                CommandManager.InvalidateRequerySuggested();
            }
        }

        public bool IsFileViewActive => CurrentView == AppView.FileCompare;
        public bool IsFolderViewActive => CurrentView == AppView.FolderCompare;

        public string Theme
        {
            get => _settings.Theme;
            set
            {
                if (string.Equals(_settings.Theme, "light", StringComparison.Ordinal))
                {
                    return;
                }

                _settings.Theme = "light";
                FileCompare.Theme = "light";
                FolderCompare.Theme = "light";
                OnPropertyChanged();
            }
        }

        public bool IsMyersEngine => FileCompare.SelectedDiffEngine == DiffEngineType.Myers;
        public bool IsSmartEngine => FileCompare.SelectedDiffEngine == DiffEngineType.Smart;

        public string StatusMessage
        {
            get => _statusMessage;
            set
            {
                if (_statusMessage == value)
                {
                    return;
                }

                _statusMessage = value;
                OnPropertyChanged();
            }
        }

        public bool CanBackToFolder
        {
            get => _canBackToFolder;
            private set
            {
                if (_canBackToFolder == value)
                {
                    return;
                }

                _canBackToFolder = value;
                OnPropertyChanged();
                CommandManager.InvalidateRequerySuggested();
            }
        }

        public string? LastLeftPath
        {
            get => _settings.LastLeftPath;
            private set => _settings.LastLeftPath = value;
        }

        public string? LastRightPath
        {
            get => _settings.LastRightPath;
            private set => _settings.LastRightPath = value;
        }

        public AppSettings Settings => _settings;

        public void OpenFiles()
        {
            var dialog = new OpenFilesDialog(LastLeftPath, LastRightPath)
            {
                Owner = Application.Current.MainWindow
            };

            if (dialog.ShowDialog() == true)
            {
                FileCompare.LoadLeftFile(dialog.LeftPath);
                FileCompare.LoadRightFile(dialog.RightPath);

                LastLeftPath = dialog.LeftPath;
                LastRightPath = dialog.RightPath;

                StatusMessage = FileCompare.StatusInfo;
                CurrentView = AppView.FileCompare;
            }
        }

        public void OpenFolders()
        {
            using var dialogLeft = new System.Windows.Forms.FolderBrowserDialog
            {
                Description = "Select Left Folder",
                SelectedPath = !string.IsNullOrWhiteSpace(LastLeftPath) ? LastLeftPath : Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)
            };

            if (dialogLeft.ShowDialog() != System.Windows.Forms.DialogResult.OK)
            {
                return;
            }

            using var dialogRight = new System.Windows.Forms.FolderBrowserDialog
            {
                Description = "Select Right Folder",
                SelectedPath = !string.IsNullOrWhiteSpace(LastRightPath) ? LastRightPath : Environment.GetFolderPath(Environment.SpecialFolder.UserProfile)
            };

            if (dialogRight.ShowDialog() != System.Windows.Forms.DialogResult.OK)
            {
                return;
            }

            var leftDir = new DirectoryInfo(dialogLeft.SelectedPath);
            var rightDir = new DirectoryInfo(dialogRight.SelectedPath);

            LastLeftPath = leftDir.FullName;
            LastRightPath = rightDir.FullName;
            _navigationStack.Clear();
            CanBackToFolder = false;

            FolderCompare.SetPaths(leftDir, rightDir);
            CurrentView = AppView.FolderCompare;
            StatusMessage = $"Comparing folders: {leftDir.FullName} <-> {rightDir.FullName}";
        }

        public void NavigateIntoFolder(FolderItemModel item)
        {
            if (item.LeftPath == null || item.RightPath == null)
            {
                return;
            }

            if (FolderCompare.LeftPath != null && FolderCompare.RightPath != null)
            {
                _navigationStack.Push((FolderCompare.LeftPath, FolderCompare.RightPath));
            }

            FolderCompare.SetPaths(item.LeftPath, item.RightPath);
            StatusMessage = $"In: {item.Name}";
        }

        public void CompareFileFromFolder(FolderItemModel item)
        {
            if (item.LeftFile == null || item.RightFile == null)
            {
                return;
            }

            FileCompare.LoadLeftFile(item.LeftFile.FullName);
            FileCompare.LoadRightFile(item.RightFile.FullName);
            CurrentView = AppView.FileCompare;
            StatusMessage = FileCompare.StatusInfo;
            CanBackToFolder = true;
        }

        public void BackToFolder()
        {
            if (!ConfirmProceedWithUnsavedChanges("Back to Folder View"))
            {
                return;
            }

            CurrentView = AppView.FolderCompare;
            CanBackToFolder = false;
        }

        public void RestorePreviousFolder()
        {
            if (_navigationStack.Count == 0)
            {
                return;
            }

            var (left, right) = _navigationStack.Pop();
            FolderCompare.SetPaths(left, right);
            StatusMessage = $"Back to: {left.Name} <-> {right.Name}";
        }

        public bool ConfirmProceedWithUnsavedChanges(string actionName)
        {
            if (!FileCompare.IsLeftModified && !FileCompare.IsRightModified)
            {
                return true;
            }

            var result = MessageBox.Show(
                $"You have unsaved changes. Save before {actionName.ToLower()}?",
                "Unsaved Changes",
                MessageBoxButton.YesNoCancel,
                MessageBoxImage.Warning);

            if (result == MessageBoxResult.Cancel)
            {
                return false;
            }

            if (result == MessageBoxResult.Yes)
            {
                SaveModifiedFiles();
            }

            return true;
        }

        public void SaveModifiedFiles()
        {
            var saved = false;
            if (FileCompare.IsLeftModified)
            {
                saved |= FileCompare.SaveLeftFile();
            }
            if (FileCompare.IsRightModified)
            {
                saved |= FileCompare.SaveRightFile();
            }

            StatusMessage = saved ? "Files saved" : "No changes to save";
        }

        public void OpenFindDialog()
        {
            if (CurrentView != AppView.FileCompare)
            {
                return;
            }

            if (_searchDialog == null)
            {
                _searchDialog = new SearchDialog(FileCompare)
                {
                    Owner = Application.Current.MainWindow
                };
                _searchDialog.Closed += (_, _) => _searchDialog = null;
            }

            _searchDialog.Show();
            _searchDialog.Activate();
        }

        public void OpenGoToLineDialog()
        {
            if (CurrentView != AppView.FileCompare)
            {
                return;
            }

            var dialog = new GoToLineDialog(FileCompare)
            {
                Owner = Application.Current.MainWindow
            };

            dialog.ShowDialog();
        }

        public void OpenFontDialog()
        {
            using var dialog = new System.Windows.Forms.FontDialog
            {
                ShowApply = false,
                ShowColor = false
            };

            var currentFamily = FileCompare.DiffFontFamily?.Source ?? "Consolas";
            var currentSize = FileCompare.DiffFontSize > 0 ? FileCompare.DiffFontSize : 13;
            var currentPoints = currentSize * 72.0 / 96.0;

            try
            {
                dialog.Font = new System.Drawing.Font(currentFamily, (float)currentPoints);
            }
            catch
            {
                dialog.Font = new System.Drawing.Font("Consolas", 13f);
            }

            if (dialog.ShowDialog() != System.Windows.Forms.DialogResult.OK)
            {
                return;
            }

            var selectedFamily = dialog.Font.FontFamily.Name;
            var selectedSize = dialog.Font.Size * 96.0 / 72.0;

            FileCompare.DiffFontFamily = new System.Windows.Media.FontFamily(selectedFamily);
            FileCompare.DiffFontSize = Math.Max(8, selectedSize);

            _settings.FontFamily = selectedFamily;
            _settings.FontSize = FileCompare.DiffFontSize;
            SettingsStore.Save(_settings);
        }

        public void Exit()
        {
            Application.Current.Shutdown();
        }

        public void About()
        {
            MessageBox.Show(
                "SmartMerge\nFile and folder comparison with merge support.",
                "About SmartMerge",
                MessageBoxButton.OK,
                MessageBoxImage.Information);
        }

        public event PropertyChangedEventHandler? PropertyChanged;

        protected void OnPropertyChanged([CallerMemberName] string? name = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
        }
    }
}
