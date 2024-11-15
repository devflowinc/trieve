# ImageConfig

Configuration for sending images to the llm

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**images_per_chunk** | **int** | The number of Images to send to the llm per chunk that is fetched more images may slow down llm inference time. default: 5 | [optional] 
**use_images** | **bool** | This sends images to the llm if chunk_metadata.image_urls has some value, the call will error if the model is not a vision LLM model. default: false | [optional] 

## Example

```python
from trieve_py_client.models.image_config import ImageConfig

# TODO update the JSON string below
json = "{}"
# create an instance of ImageConfig from a JSON string
image_config_instance = ImageConfig.from_json(json)
# print the JSON string representation of the object
print(ImageConfig.to_json())

# convert the object into a dict
image_config_dict = image_config_instance.to_dict()
# create an instance of ImageConfig from a dict
image_config_form_dict = image_config.from_dict(image_config_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


