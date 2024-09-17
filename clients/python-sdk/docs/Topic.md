# Topic


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**dataset_id** | **str** |  | 
**deleted** | **bool** |  | 
**id** | **str** |  | 
**name** | **str** |  | 
**owner_id** | **str** |  | 
**updated_at** | **datetime** |  | 

## Example

```python
from trieve_py_client.models.topic import Topic

# TODO update the JSON string below
json = "{}"
# create an instance of Topic from a JSON string
topic_instance = Topic.from_json(json)
# print the JSON string representation of the object
print(Topic.to_json())

# convert the object into a dict
topic_dict = topic_instance.to_dict()
# create an instance of Topic from a dict
topic_form_dict = topic.from_dict(topic_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


