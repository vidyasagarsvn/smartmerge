namespace SmartMerge.Models
{
    /// <summary>
    /// Represents a file comparison result
    /// </summary>
    public class FileComparisonModel
    {
        public string? LeftFilePath { get; set; }
        public string? RightFilePath { get; set; }
        public string? LeftContent { get; set; }
        public string? RightContent { get; set; }
        public List<DiffLine>? DiffLines { get; set; }
        public bool IsIdentical { get; set; }
    }

    /// <summary>
    /// Represents a single line in a diff
    /// </summary>
    public class DiffLine
    {
        public int LeftLineNumber { get; set; }
        public int RightLineNumber { get; set; }
        public string LeftContent { get; set; } = string.Empty;
        public string RightContent { get; set; } = string.Empty;
        public DiffLineType Type { get; set; }
    }

    /// <summary>
    /// Type of diff line
    /// </summary>
    public enum DiffLineType
    {
        Unchanged,
        Added,
        Removed,
        Modified,
        Conflict
    }
}
