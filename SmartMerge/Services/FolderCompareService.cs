using System.Collections.Generic;
using System.IO;
using System.Linq;
using SmartMerge.Models;

namespace SmartMerge.Services
{
    public static class FolderCompareService
    {
        public static List<FolderItemModel> CompareFolders(DirectoryInfo leftPath, DirectoryInfo rightPath)
        {
            var results = new List<FolderItemModel>();
            if (!leftPath.Exists || !rightPath.Exists)
            {
                return results;
            }

            var leftItems = leftPath.EnumerateFileSystemInfos().ToDictionary(p => p.Name, p => p);
            var rightItems = rightPath.EnumerateFileSystemInfos().ToDictionary(p => p.Name, p => p);

            var allNames = new HashSet<string>(leftItems.Keys);
            foreach (var name in rightItems.Keys)
            {
                allNames.Add(name);
            }

            foreach (var name in allNames.OrderBy(n => n))
            {
                leftItems.TryGetValue(name, out var leftItem);
                rightItems.TryGetValue(name, out var rightItem);

                if (leftItem != null && rightItem != null)
                {
                    if (leftItem is DirectoryInfo leftDir && rightItem is DirectoryInfo rightDir)
                    {
                        results.Add(new FolderItemModel
                        {
                            Name = name,
                            Type = FolderItemType.Folder,
                            Status = FolderItemStatus.Identical,
                            LeftPath = leftDir,
                            RightPath = rightDir
                        });
                    }
                    else if (leftItem is FileInfo leftFile && rightItem is FileInfo rightFile)
                    {
                        results.Add(new FolderItemModel
                        {
                            Name = name,
                            Type = FolderItemType.File,
                            Status = CompareFiles(leftFile, rightFile),
                            LeftFile = leftFile,
                            RightFile = rightFile
                        });
                    }
                    else
                    {
                        results.Add(new FolderItemModel
                        {
                            Name = name,
                            Type = FolderItemType.File,
                            Status = FolderItemStatus.Modified
                        });
                    }
                }
                else if (leftItem != null)
                {
                    if (leftItem is DirectoryInfo leftDir)
                    {
                        results.Add(new FolderItemModel
                        {
                            Name = name,
                            Type = FolderItemType.Folder,
                            Status = FolderItemStatus.FolderLeftOnly,
                            LeftPath = leftDir
                        });
                    }
                    else if (leftItem is FileInfo leftFile)
                    {
                        results.Add(new FolderItemModel
                        {
                            Name = name,
                            Type = FolderItemType.File,
                            Status = FolderItemStatus.AddedLeft,
                            LeftFile = leftFile
                        });
                    }
                }
                else if (rightItem != null)
                {
                    if (rightItem is DirectoryInfo rightDir)
                    {
                        results.Add(new FolderItemModel
                        {
                            Name = name,
                            Type = FolderItemType.Folder,
                            Status = FolderItemStatus.FolderRightOnly,
                            RightPath = rightDir
                        });
                    }
                    else if (rightItem is FileInfo rightFile)
                    {
                        results.Add(new FolderItemModel
                        {
                            Name = name,
                            Type = FolderItemType.File,
                            Status = FolderItemStatus.AddedRight,
                            RightFile = rightFile
                        });
                    }
                }
            }

            return results;
        }

        public static bool IsFolderNavigable(FolderItemModel item)
        {
            return item.Type == FolderItemType.Folder && item.LeftPath != null && item.RightPath != null;
        }

        private static FolderItemStatus CompareFiles(FileInfo leftFile, FileInfo rightFile)
        {
            string leftContent;
            string rightContent;

            try
            {
                leftContent = File.ReadAllText(leftFile.FullName);
            }
            catch
            {
                return FolderItemStatus.Modified;
            }

            try
            {
                rightContent = File.ReadAllText(rightFile.FullName);
            }
            catch
            {
                return FolderItemStatus.Modified;
            }

            if (leftContent == rightContent)
            {
                return FolderItemStatus.Identical;
            }

            var leftLines = leftContent.Split(new[] { "\r\n", "\r", "\n" }, System.StringSplitOptions.None);
            var rightLines = rightContent.Split(new[] { "\r\n", "\r", "\n" }, System.StringSplitOptions.None);
            var opcodes = DiffEngine.MyersOpcodes(leftLines, rightLines);
            return opcodes.Count == 0 ? FolderItemStatus.Identical : FolderItemStatus.Modified;
        }
    }
}
