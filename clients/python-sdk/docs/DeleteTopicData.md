# DeleteTopicData


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**topic_id** | **str** | The id of the topic to target. | 

## Example

```python
from trieve_py_client.models.delete_topic_data import DeleteTopicData

# TODO update the JSON string below
json = "{}"
# create an instance of DeleteTopicData from a JSON string
delete_topic_data_instance = DeleteTopicData.from_json(json)
# print the JSON string representation of the object
print(DeleteTopicData.to_json())

# convert the object into a dict
delete_topic_data_dict = delete_topic_data_instance.to_dict()
# create an instance of DeleteTopicData from a dict
delete_topic_data_form_dict = delete_topic_data.from_dict(delete_topic_data_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


