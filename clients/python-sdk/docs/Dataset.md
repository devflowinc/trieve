# Dataset


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** | Timestamp of the creation of the dataset | 
**deleted** | **int** | Flag to indicate if the dataset has been deleted. Deletes are handled async after the flag is set so as to avoid expensive search index compaction. | 
**id** | **str** | Unique identifier of the dataset, auto-generated uuid created by Trieve | 
**name** | **str** | Name of the dataset | 
**organization_id** | **str** | Unique identifier of the organization that owns the dataset | 
**server_configuration** | **object** | Configuration of the dataset for RAG, embeddings, BM25, etc. | 
**tracking_id** | **str** | Tracking ID of the dataset, can be any string, determined by the user. Tracking ID&#39;s are unique identifiers for datasets within an organization. They are designed to match the unique identifier of the dataset in the user&#39;s system. | [optional] 
**updated_at** | **datetime** | Timestamp of the last update of the dataset | 

## Example

```python
from trieve_py_client.models.dataset import Dataset

# TODO update the JSON string below
json = "{}"
# create an instance of Dataset from a JSON string
dataset_instance = Dataset.from_json(json)
# print the JSON string representation of the object
print(Dataset.to_json())

# convert the object into a dict
dataset_dict = dataset_instance.to_dict()
# create an instance of Dataset from a dict
dataset_form_dict = dataset.from_dict(dataset_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


