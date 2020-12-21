# dupsearch

Find duplicate files in a specified directory and output them in a specified format. You can select JSON, YAML and TOML as the format.

## Usage

```
// Find duplicate files in the current directory and output them in JSON format. 
// The Blake3 algorithm is used to detect duplicates.
$ dupsearch
```


```
// Output only those files in the "hoge" directory that had more than 4 and less than 10 duplicates in YAML format. 
// The MD5 algorithm is used to detect duplicates.
$ dupsearch hoge --min 4 --max 10 -f YAML -a MD5
```
