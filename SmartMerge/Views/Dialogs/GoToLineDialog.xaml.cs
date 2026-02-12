using System;
using System.Windows;
using SmartMerge.ViewModels;

namespace SmartMerge.Views.Dialogs
{
    public partial class GoToLineDialog : Window
    {
        private readonly FileCompareViewModel _viewModel;

        public GoToLineDialog(FileCompareViewModel viewModel)
        {
            InitializeComponent();
            _viewModel = viewModel;
            Loaded += (_, _) => LineBox.Focus();
        }

        private void OnGo(object sender, RoutedEventArgs e)
        {
            if (int.TryParse(LineBox.Text, out var lineNumber) && lineNumber > 0)
            {
                _viewModel.GoToLine(lineNumber - 1);
                Close();
            }
        }

        private void OnCancel(object sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}
