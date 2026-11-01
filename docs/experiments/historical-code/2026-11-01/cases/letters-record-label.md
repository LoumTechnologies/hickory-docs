# letters-record-label

Implement LabelImage. Parse the integer captured by _imageFileNameRegex from the basename of ImagePath.Value. Append a ManualClassification with that X and the current Letter.Value, then call the supplied PrepareToClassifyNextImage. Assume a valid matching image path. Keep the label string exactly as entered.

## Historical solution

Retained from [nloum/letter-classifier at c97dd60d4a50](https://github.com/nloum/letter-classifier/blob/c97dd60d4a5061e5086f7c7e3db74ad6593d0afe/MainViewModel.cs#L92). This is a method projection with the [fixture limits](../../../../developers/historical-code-benchmark.md); the prompt is retrospective.

```csharp
        private void LabelImage()
        {
            var match = _imageFileNameRegex.Match(Path.GetFileName(ImagePath.Value));
            var capture = int.Parse(match.Groups[1].Value);

            manualClassifications.Add(new ManualClassification
            {
                Letter = Letter.Value,
                X = capture
            });

            PrepareToClassifyNextImage();
        }
```

## Public acceptance example

```csharp
var s=new Subject(); s.xValues.AddRange(new[]{7,4}); s.ImagePath.Value=@"images\image7.png"; s.Letter.Value="Q"; s.LabelImage(); Equal(s.manualClassifications.Count,1); Equal(s.manualClassifications[0].X,7); Equal(s.manualClassifications[0].Letter,"Q"); Equal(s.ImagePath.Value,@"images\image4.png");
```
