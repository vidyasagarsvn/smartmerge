using System.Windows;
using System.Windows.Input;
using SmartMerge.ViewModels;

namespace SmartMerge.Views.Dialogs
{
    public partial class SearchDialog : Window
    {
        private readonly FileCompareViewModel _viewModel;

        public SearchDialog(FileCompareViewModel viewModel)
        {
            InitializeComponent();
            _viewModel = viewModel;
            Loaded += (_, _) => SearchBox.Focus();
        }

        private void OnFindNext(object sender, RoutedEventArgs e)
        {
            ApplySearch();
            _viewModel.FindNext();
        }

        private void OnFindPrevious(object sender, RoutedEventArgs e)
        {
            ApplySearch();
            _viewModel.FindPrevious();
        }

        private void OnClose(object sender, RoutedEventArgs e)
        {
            Close();
        }

        private void OnSearchKeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key == Key.Enter)
            {
                ApplySearch();
                _viewModel.FindNext();
            }
        }

        private void ApplySearch()
        {
            var text = SearchBox.Text ?? string.Empty;
            if (string.IsNullOrWhiteSpace(text))
            {
                return;
            }

            _viewModel.SetSearch(text, CaseSensitiveBox.IsChecked == true);
        }
    }
}
