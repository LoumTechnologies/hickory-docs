# codeio-cartesian-product

Implement a lazy Cartesian product over restartable finite sequences. Each result is an ImmutableList<TItem>. The first input dimension advances fastest. Empty outer input yields one empty combination. Any empty inner sequence throws ArgumentException when the result is enumerated. Do not change the signature.

## Historical solution

Retained from [nloum/CodeIO at a5640ad8755b](https://github.com/nloum/CodeIO/blob/a5640ad8755b3a2f0cbbfbbf9d12d97ac1f56631/FluentSourceGenerators/Utilities.cs#L216). This is a method projection with the [fixture limits](../../../../developers/historical-code-benchmark.md); the prompt is retrospective.

```csharp
        public static IEnumerable<ImmutableList<TItem>> CalcCombinationsOfOneFromEach<TItem>(
            IEnumerable<IEnumerable<TItem>> enumerables ) {
            var enumerators = enumerables.Select( enumerable => Tuple.Create( enumerable, enumerable.GetEnumerator() ) ).ToList();

            foreach ( var enumerator in enumerators ) {
                if ( !enumerator.Item2.MoveNext() ) { throw new ArgumentException( "An empty enumerable was specified" );
                }
            }

            yield return enumerators.Select( t => t.Item2.Current ).ToImmutableList();

            while (true) {
                var i = 0;
                while(true) {
                    if ( i == enumerators.Count ) {
                        yield break;
                    }

                    if ( !enumerators[i].Item2.MoveNext() ) {
                        enumerators[i] = Tuple.Create( enumerators[i].Item1,
                            enumerators[i].Item1.GetEnumerator() );
                        enumerators[i].Item2.MoveNext();
                    } else {
                        break;
                    }

                    i++;
                }

                yield return enumerators.Select( t => t.Item2.Current ).ToImmutableList();
            }
        }
```

## Public acceptance example

```csharp
var got=CalcCombinationsOfOneFromEach(new[]{new[]{1,2},new[]{3,4}}).Select(x=>string.Join(",",x)); Seq(got,"1,3","2,3","1,4","2,4");
```
