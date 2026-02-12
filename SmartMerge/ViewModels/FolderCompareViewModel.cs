using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using SmartMerge.Models;
using SmartMerge.Services;

namespace SmartMerge.ViewModels
{
    public class FolderCompareViewModel : INotifyPropertyChanged
    {
        private DirectoryInfo? _leftPath;
        private DirectoryInfo? _rightPath;
        private bool _filterIdentical = true;
        private bool _filterModified = true;
        private bool _filterLeftOnly = true;
        private bool _filterRightOnly = true;
        private bool _filterMissing = true;
        private string _theme = "light";

        public ObservableCollection<FolderItemModel> Items { get; } = new();

        public DirectoryInfo? LeftPath
        {
            get => _leftPath;
            set
            {
                if (_leftPath == value)
                {
                    return;
                }

                _leftPath = value;
                OnPropertyChanged();
            }
        }

        public DirectoryInfo? RightPath
        {
            get => _rightPath;
            set
            {
                if (_rightPath == value)
                {
                    return;
                }

                _rightPath = value;
                OnPropertyChanged();
            }
        }

        public bool FilterIdentical
        {
            get => _filterIdentical;
            set
            {
                if (_filterIdentical == value)
                {
                    return;
                }

                _filterIdentical = value;
                OnPropertyChanged();
                RefreshItems();
            }
        }

        public bool FilterModified
        {
            get => _filterModified;
            set
            {
                if (_filterModified == value)
                {
                    return;
                }

                _filterModified = value;
                OnPropertyChanged();
                RefreshItems();
            }
        }

        public bool FilterLeftOnly
        {
            get => _filterLeftOnly;
            set
            {
                if (_filterLeftOnly == value)
                {
                    return;
                }

                _filterLeftOnly = value;
                OnPropertyChanged();
                RefreshItems();
            }
        }

        public bool FilterRightOnly
        {
            get => _filterRightOnly;
            set
            {
                if (_filterRightOnly == value)
                {
                    return;
                }

                _filterRightOnly = value;
                OnPropertyChanged();
                RefreshItems();
            }
        }

        public bool FilterMissing
        {
            get => _filterMissing;
            set
            {
                if (_filterMissing == value)
                {
                    return;
                }

                _filterMissing = value;
                OnPropertyChanged();
                RefreshItems();
            }
        }

        public string Theme
        {
            get => _theme;
            set
            {
                if (_theme == value)
                {
                    return;
                }

                _theme = value;
                OnPropertyChanged();
            }
        }

        public void SetPaths(DirectoryInfo leftPath, DirectoryInfo rightPath)
        {
            LeftPath = leftPath;
            RightPath = rightPath;
            RefreshItems();
        }

        public void RefreshItems()
        {
            Items.Clear();
            if (LeftPath == null || RightPath == null)
            {
                return;
            }

            TryAddParentItem();

            var results = FolderCompareService.CompareFolders(LeftPath, RightPath);
            foreach (var item in results.Where(ShouldInclude))
            {
                Items.Add(item);
            }
        }

        private void TryAddParentItem()
        {
            try
            {
                var leftParent = LeftPath?.Parent;
                var rightParent = RightPath?.Parent;
                if (leftParent != null && rightParent != null)
                {
                    Items.Add(new FolderItemModel
                    {
                        Name = "..",
                        Type = FolderItemType.Folder,
                        Status = FolderItemStatus.Identical,
                        LeftPath = leftParent,
                        RightPath = rightParent
                    });
                }
            }
            catch
            {
                // Ignore invalid path errors.
            }
        }

        public bool IsNavigable(FolderItemModel item)
        {
            return FolderCompareService.IsFolderNavigable(item);
        }

        public bool ShouldInclude(FolderItemModel item)
        {
            return item.Status switch
            {
                FolderItemStatus.Identical => FilterIdentical,
                FolderItemStatus.Modified => FilterModified,
                FolderItemStatus.AddedLeft => FilterLeftOnly,
                FolderItemStatus.FolderLeftOnly => FilterLeftOnly,
                FolderItemStatus.AddedRight => FilterRightOnly,
                FolderItemStatus.FolderRightOnly => FilterRightOnly,
                FolderItemStatus.DeletedLeft => FilterMissing,
                FolderItemStatus.DeletedRight => FilterMissing,
                _ => true
            };
        }

        public event PropertyChangedEventHandler? PropertyChanged;

        protected void OnPropertyChanged([CallerMemberName] string? name = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(name));
        }
    }
}
