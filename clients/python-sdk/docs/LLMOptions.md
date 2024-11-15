# LLMOptions

LLM options to use for the completion. If not specified, this defaults to the dataset's LLM options.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**completion_first** | **bool** | Completion first decides whether the stream should contain the stream of the completion response or the chunks first. Default is false. Keep in mind that || is used to separate the chunks from the completion response. If || is in the completion then you may want to split on ||{ instead. | [optional] 
**frequency_penalty** | **float** | Frequency penalty is a number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model&#39;s likelihood to repeat the same line verbatim. Default is 0.7. | [optional] 
**image_config** | [**ImageConfig**](ImageConfig.md) |  | [optional] 
**max_tokens** | **int** | The maximum number of tokens to generate in the chat completion. Default is None. | [optional] 
**presence_penalty** | **float** | Presence penalty is a number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model&#39;s likelihood to talk about new topics. Default is 0.7. | [optional] 
**stop_tokens** | **List[str]** | Stop tokens are up to 4 sequences where the API will stop generating further tokens. Default is None. | [optional] 
**stream_response** | **bool** | Whether or not to stream the response. If this is set to true or not included, the response will be a stream. If this is set to false, the response will be a normal JSON response. Default is true. | [optional] 
**system_prompt** | **str** | Optionally, override the system prompt in dataset server settings. | [optional] 
**temperature** | **float** | What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. Default is 0.5. | [optional] 

## Example

```python
from trieve_py_client.models.llm_options import LLMOptions

# TODO update the JSON string below
json = "{}"
# create an instance of LLMOptions from a JSON string
llm_options_instance = LLMOptions.from_json(json)
# print the JSON string representation of the object
print(LLMOptions.to_json())

# convert the object into a dict
llm_options_dict = llm_options_instance.to_dict()
# create an instance of LLMOptions from a dict
llm_options_form_dict = llm_options.from_dict(llm_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


