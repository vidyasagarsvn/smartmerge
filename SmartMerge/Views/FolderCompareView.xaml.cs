using System;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using SmartMerge.Models;
using SmartMerge.ViewModels;

namespace SmartMerge.Views
{
    public partial class FolderCompareView : UserControl
    {
        public event EventHandler<FolderItemModel>? FolderSelected;
        public event EventHandler<FolderItemModel>? FileSelected;

        public FolderCompareView()
        {
            InitializeComponent();
            ItemsGrid.MouseDoubleClick += OnMouseDoubleClick;
        }

        private void OnMouseDoubleClick(object sender, MouseButtonEventArgs e)
        {
            if (ItemsGrid.SelectedItem is not FolderItemModel item)
            {
                return;
            }

            if (DataContext is not FolderCompareViewModel vm)
            {
                return;
            }

            if (item.Type == FolderItemType.Folder && vm.IsNavigable(item))
            {
                FolderSelected?.Invoke(this, item);
            }
            else if (item.Type == FolderItemType.File && item.LeftFile != null && item.RightFile != null)
            {
                FileSelected?.Invoke(this, item);
            }
        }
    }
}
