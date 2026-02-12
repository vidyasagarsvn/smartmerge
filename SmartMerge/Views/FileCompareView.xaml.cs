using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using SmartMerge.ViewModels;

namespace SmartMerge.Views
{
    public partial class FileCompareView : UserControl
    {
        private bool _isSyncingScroll;

        public FileCompareView()
        {
            InitializeComponent();
            Loaded += OnLoaded;
            DataContextChanged += OnDataContextChanged;
        }

        private void OnLoaded(object sender, RoutedEventArgs e)
        {
            HookScrollSync();
        }

        private void OnDataContextChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            if (e.OldValue is FileCompareViewModel oldVm)
            {
                oldVm.ScrollToRowRequested -= OnScrollToRowRequested;
            }

            if (e.NewValue is FileCompareViewModel newVm)
            {
                newVm.ScrollToRowRequested += OnScrollToRowRequested;
            }
        }

        private void OnScrollToRowRequested(object? sender, int rowIndex)
        {
            if (rowIndex < 0)
            {
                return;
            }

            var row = LeftList.Items.Count > rowIndex ? LeftList.Items[rowIndex] : null;
            if (row != null)
            {
                LeftList.ScrollIntoView(row);
                RightList.ScrollIntoView(row);
            }
        }

        private void HookScrollSync()
        {
            var leftScrollViewer = FindScrollViewer(LeftList);
            var rightScrollViewer = FindScrollViewer(RightList);
            if (leftScrollViewer == null || rightScrollViewer == null)
            {
                return;
            }

            leftScrollViewer.ScrollChanged += (s, e) => SyncScroll(leftScrollViewer, rightScrollViewer, e);
            rightScrollViewer.ScrollChanged += (s, e) => SyncScroll(rightScrollViewer, leftScrollViewer, e);
        }

        private void SyncScroll(ScrollViewer source, ScrollViewer target, ScrollChangedEventArgs args)
        {
            if (_isSyncingScroll)
            {
                return;
            }

            _isSyncingScroll = true;
            try
            {
                if (args.VerticalChange != 0)
                {
                    target.ScrollToVerticalOffset(source.VerticalOffset);
                }
            }
            finally
            {
                _isSyncingScroll = false;
            }
        }

        private static ScrollViewer? FindScrollViewer(DependencyObject root)
        {
            if (root is ScrollViewer viewer)
            {
                return viewer;
            }

            for (var i = 0; i < VisualTreeHelper.GetChildrenCount(root); i++)
            {
                var child = VisualTreeHelper.GetChild(root, i);
                var result = FindScrollViewer(child);
                if (result != null)
                {
                    return result;
                }
            }

            return null;
        }
    }
}
