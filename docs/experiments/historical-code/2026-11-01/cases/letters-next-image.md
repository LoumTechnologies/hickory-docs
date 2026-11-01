# letters-next-image

Implement PrepareToClassifyNextImage. Visit xValues in their existing order; choose the first X without a manual classification, set ImagePath.Value to _folderPath + "\\image" + X + ".png", clear Letter.Value, and stop. If all values are classified (or the list is empty), leave both properties unchanged. Use the supplied small collaborators.

## Historical solution

Retained from [nloum/letter-classifier at c97dd60d4a50](https://github.com/nloum/letter-classifier/blob/c97dd60d4a5061e5086f7c7e3db74ad6593d0afe/MainViewModel.cs#L47). This is a method projection with the [fixture limits](../../../../developers/historical-code-benchmark.md); the prompt is retrospective.

```csharp
        private void PrepareToClassifyNextImage()
        {
            foreach(var x in xValues)
            {
                if (!manualClassifications.Any(c => c.X == x))
                {
                    ImagePath.Value = _folderPath + "\\image" + x + ".png";
                    Letter.Value = "";
                    break;
                }
            }
        }
```

## Public acceptance example

```csharp
var s=new Subject(); s.xValues.AddRange(new[]{9,2}); s.PrepareToClassifyNextImage(); Equal(s.ImagePath.Value,@"images\image9.png"); Equal(s.Letter.Value,"");
```
