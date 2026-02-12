using System;
using System.Globalization;
using System.Windows;
using System.Windows.Data;
using System.Windows.Media;
using SmartMerge.Models;
using SmartMerge.ViewModels;

namespace SmartMerge.Utils
{
    public class BoolToVisibilityConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is bool flag && flag)
            {
                return Visibility.Visible;
            }

            return Visibility.Collapsed;
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            return value is Visibility visibility && visibility == Visibility.Visible;
        }
    }

    public class DiffKindToBrushConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is not DiffLineKind kind)
            {
                return Brushes.Transparent;
            }

            return kind switch
            {
                DiffLineKind.Added => GetBrush("DiffAddedBrush"),
                DiffLineKind.Removed => GetBrush("DiffRemovedBrush"),
                DiffLineKind.Modified => GetBrush("DiffModifiedBrush"),
                _ => Brushes.Transparent
            };
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotSupportedException();
        }

        private static Brush GetBrush(string key)
        {
            return Application.Current.TryFindResource(key) as Brush ?? Brushes.Transparent;
        }
    }

    public class FolderStatusToTextConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (value is not FolderItemStatus status)
            {
                return string.Empty;
            }

            return status switch
            {
                FolderItemStatus.Identical => "Identical",
                FolderItemStatus.Modified => "Modified",
                FolderItemStatus.AddedLeft => "Left Only",
                FolderItemStatus.AddedRight => "Right Only",
                FolderItemStatus.DeletedLeft => "Left Missing",
                FolderItemStatus.DeletedRight => "Right Missing",
                FolderItemStatus.FolderLeftOnly => "Left Only",
                FolderItemStatus.FolderRightOnly => "Right Only",
                _ => "Unknown"
            };
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotSupportedException();
        }
    }

    public class FolderNameConverter : IMultiValueConverter
    {
        public object Convert(object[] values, Type targetType, object parameter, CultureInfo culture)
        {
            if (values.Length < 2)
            {
                return string.Empty;
            }

            var name = values[0] as string ?? string.Empty;
            var type = values[1] is FolderItemType folderType ? folderType : FolderItemType.File;
            var prefix = type == FolderItemType.Folder ? "[DIR] " : "[FILE] ";
            return prefix + name;
        }

        public object[] ConvertBack(object value, Type[] targetTypes, object parameter, CultureInfo culture)
        {
            throw new NotSupportedException();
        }
    }
}
