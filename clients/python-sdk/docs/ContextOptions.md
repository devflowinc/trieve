# ContextOptions

Context options to use for the completion. If not specified, all options will default to false.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**include_links** | **bool** | Include links in the context. If not specified, this defaults to false. | [optional] 

## Example

```python
from trieve_py_client.models.context_options import ContextOptions

# TODO update the JSON string below
json = "{}"
# create an instance of ContextOptions from a JSON string
context_options_instance = ContextOptions.from_json(json)
# print the JSON string representation of the object
print(ContextOptions.to_json())

# convert the object into a dict
context_options_dict = context_options_instance.to_dict()
# create an instance of ContextOptions from a dict
context_options_form_dict = context_options.from_dict(context_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


