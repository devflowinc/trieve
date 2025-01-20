# MmrOptions

MMR Options lets you specify different methods to rerank the chunks in the result set using Maximal Marginal Relevance. If not specified, this defaults to the score of the chunks.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**mmr_lambda** | **float** | Set mmr_lambda to a value between 0.0 and 1.0 to control the tradeoff between relevance and diversity. Closer to 1.0 will give more diverse results, closer to 0.0 will give more relevant results. If not specified, this defaults to 0.5. | [optional] 
**use_mmr** | **bool** | Set use_mmr to true to use the Maximal Marginal Relevance algorithm to rerank the results. | 

## Example

```python
from trieve_py_client.models.mmr_options import MmrOptions

# TODO update the JSON string below
json = "{}"
# create an instance of MmrOptions from a JSON string
mmr_options_instance = MmrOptions.from_json(json)
# print the JSON string representation of the object
print(MmrOptions.to_json())

# convert the object into a dict
mmr_options_dict = mmr_options_instance.to_dict()
# create an instance of MmrOptions from a dict
mmr_options_form_dict = mmr_options.from_dict(mmr_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


