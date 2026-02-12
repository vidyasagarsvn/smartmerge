using System.IO;

namespace SmartMerge.Models
{
    public enum FolderItemType
    {
        File,
        Folder
    }

    public enum FolderItemStatus
    {
        Identical,
        Modified,
        AddedLeft,
        AddedRight,
        DeletedLeft,
        DeletedRight,
        FolderLeftOnly,
        FolderRightOnly
    }

    public class FolderItemModel
    {
        public string Name { get; set; } = string.Empty;
        public FolderItemType Type { get; set; }
        public FolderItemStatus Status { get; set; }
        public DirectoryInfo? LeftPath { get; set; }
        public DirectoryInfo? RightPath { get; set; }
        public FileInfo? LeftFile { get; set; }
        public FileInfo? RightFile { get; set; }
    }
}
