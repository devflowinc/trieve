# CreateDatasetBatchReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**datasets** | [**List[CreateBatchDataset]**](CreateBatchDataset.md) | List of datasets to create | 
**upsert** | **bool** | Upsert when a dataset with one of the specified tracking_ids already exists. By default this is false and specified datasets with a tracking_id that already exists in the org will not be ignored. If true, the existing dataset will be updated with the new dataset&#39;s details. | [optional] 

## Example

```python
from trieve_py_client.models.create_dataset_batch_req_payload import CreateDatasetBatchReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of CreateDatasetBatchReqPayload from a JSON string
create_dataset_batch_req_payload_instance = CreateDatasetBatchReqPayload.from_json(json)
# print the JSON string representation of the object
print(CreateDatasetBatchReqPayload.to_json())

# convert the object into a dict
create_dataset_batch_req_payload_dict = create_dataset_batch_req_payload_instance.to_dict()
# create an instance of CreateDatasetBatchReqPayload from a dict
create_dataset_batch_req_payload_form_dict = create_dataset_batch_req_payload.from_dict(create_dataset_batch_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


