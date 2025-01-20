# Document


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**extract** | **str** |  | [optional] 
**html** | **str** |  | [optional] 
**links** | **List[str]** |  | [optional] 
**markdown** | **str** |  | [optional] 
**metadata** | [**Metadata**](Metadata.md) |  | 
**raw_html** | **str** |  | [optional] 
**screenshot** | **str** |  | [optional] 

## Example

```python
from trieve_py_client.models.document import Document

# TODO update the JSON string below
json = "{}"
# create an instance of Document from a JSON string
document_instance = Document.from_json(json)
# print the JSON string representation of the object
print(Document.to_json())

# convert the object into a dict
document_dict = document_instance.to_dict()
# create an instance of Document from a dict
document_form_dict = document.from_dict(document_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


