using System;
using System.Collections.Generic;
using System.IO;
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
        private readonly ISideBySideDiffBuilder _diffBuilder;

        public DiffService()
        {
            _diffBuilder = new SideBySideDiffBuilder(new Differ());
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
            if (diffModel == null || diffModel.OldText == null || diffModel.NewText == null)
                return diffLines;

            int maxLines = Math.Max(diffModel.OldText.Lines.Count, diffModel.NewText.Lines.Count);
            for (int i = 0; i < maxLines; i++)
            {
                var leftLine = i < diffModel.OldText.Lines.Count ? diffModel.OldText.Lines[i] : null;
                var rightLine = i < diffModel.NewText.Lines.Count ? diffModel.NewText.Lines[i] : null;

                var diffLine = new DiffLine
                {
                    LeftLineNumber = leftLine?.Position ?? 0,
                    RightLineNumber = rightLine?.Position ?? 0,
                    LeftContent = leftLine?.Text ?? string.Empty,
                    RightContent = rightLine?.Text ?? string.Empty,
                    Type = leftLine != null && rightLine != null
                        ? (leftLine.Type == ChangeType.Unchanged && rightLine.Type == ChangeType.Unchanged ? DiffLineType.Unchanged :
                            leftLine.Type == ChangeType.Deleted ? DiffLineType.Removed :
                            rightLine.Type == ChangeType.Inserted ? DiffLineType.Added : DiffLineType.Modified)
                        : (leftLine != null ? (leftLine.Type == ChangeType.Deleted ? DiffLineType.Removed : DiffLineType.Unchanged)
                            : (rightLine != null && rightLine.Type == ChangeType.Inserted ? DiffLineType.Added : DiffLineType.Unchanged))
                };
                diffLines.Add(diffLine);
            }
            return diffLines;
        }
    }
}
