using DiffPlex;
using DiffPlex.DiffBuilder;
using DiffPlex.DiffBuilder.Model;
using SmartMerge.Models;

namespace SmartMerge.Services
{
    /// <summary>
    /// Service for performing diff and merge operations
    /// </summary>
    public class DiffService
    {
        private readonly IDiffBuilder _diffBuilder;

        public DiffService()
        {
            _diffBuilder = new SideBySideDiffBuilder(new DiffMatchPatch());
        }

        /// <summary>
        /// Compares two files and returns diff results
        /// </summary>
        public FileComparisonModel CompareFiles(string leftFilePath, string rightFilePath)
        {
            try
            {
                var leftContent = File.ReadAllText(leftFilePath);
                var rightContent = File.ReadAllText(rightFilePath);

                return CompareContent(leftContent, rightContent, leftFilePath, rightFilePath);
            }
            catch (Exception ex)
            {
                throw new InvalidOperationException($"Error comparing files: {ex.Message}", ex);
            }
        }

        /// <summary>
        /// Compares two text contents
        /// </summary>
        public FileComparisonModel CompareContent(string leftContent, string rightContent, 
            string? leftLabel = null, string? rightLabel = null)
        {
            var model = new FileComparisonModel
            {
                LeftFilePath = leftLabel,
                RightFilePath = rightLabel,
                LeftContent = leftContent,
                RightContent = rightContent
            };

            // Split content into lines
            var leftLines = leftContent.Split(new[] { "\r\n", "\r", "\n" }, StringSplitOptions.None);
            var rightLines = rightContent.Split(new[] { "\r\n", "\r", "\n" }, StringSplitOptions.None);

            // Build diff model
            var diffModel = _diffBuilder.BuildDiffModel(leftContent, rightContent);

            // Convert to our diff line format
            model.DiffLines = BuildDiffLines(diffModel);
            model.IsIdentical = leftContent == rightContent;

            return model;
        }

        /// <summary>
        /// Converts DiffPlex diff model to our DiffLine format
        /// </summary>
        private List<DiffLine> BuildDiffLines(SideBySideDiffModel diffModel)
        {
            var diffLines = new List<DiffLine>();

            if (diffModel.OldSideChanges == null || diffModel.NewSideChanges == null)
                return diffLines;

            int leftLineNum = 1;
            int rightLineNum = 1;

            foreach (var line in diffModel.OldSideChanges)
            {
                var diffLine = new DiffLine
                {
                    LeftLineNumber = line.Position,
                    LeftContent = line.Text ?? string.Empty,
                    Type = line.Type switch
                    {
                        ChangeType.Deleted => DiffLineType.Removed,
                        ChangeType.Inserted => DiffLineType.Added,
                        _ => DiffLineType.Unchanged
                    }
                };
                diffLines.Add(diffLine);
            }

            return diffLines;
        }
    }
}
