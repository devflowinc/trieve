# Image


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**image_url** | **str** |  | 
**llm_prompt** | **str** |  | [optional] 

## Example

```python
from trieve_py_client.models.image import Image

# TODO update the JSON string below
json = "{}"
# create an instance of Image from a JSON string
image_instance = Image.from_json(json)
# print the JSON string representation of the object
print(Image.to_json())

# convert the object into a dict
image_dict = image_instance.to_dict()
# create an instance of Image from a dict
image_form_dict = image.from_dict(image_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


