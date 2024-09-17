# TypoOptions

Typo Options lets you specify different methods to correct typos in the query. If not specified, typos will not be corrected.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**correct_typos** | **bool** | Set correct_typos to true to correct typos in the query. If not specified, this defaults to false. | [optional] 
**disable_on_word** | **List[str]** | Words that should not be corrected. If not specified, this defaults to an empty list. | [optional] 
**one_typo_word_range** | [**TypoRange**](TypoRange.md) |  | [optional] 
**prioritize_domain_specifc_words** | **bool** | Auto-require non-english words present in the dataset to exist in each results chunk_html text. If not specified, this defaults to true. | [optional] 
**two_typo_word_range** | [**TypoRange**](TypoRange.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.typo_options import TypoOptions

# TODO update the JSON string below
json = "{}"
# create an instance of TypoOptions from a JSON string
typo_options_instance = TypoOptions.from_json(json)
# print the JSON string representation of the object
print(TypoOptions.to_json())

# convert the object into a dict
typo_options_dict = typo_options_instance.to_dict()
# create an instance of TypoOptions from a dict
typo_options_form_dict = typo_options.from_dict(typo_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


