import datetime
import re
import discord
import pytz
import logging
from src.cogs.abakus.event import Event

local_timezone = pytz.timezone("Europe/Oslo")
logger = logging.getLogger(__name__)

def generate_message(event_object:Event, template:str):
    if event_object == None:
        raise ValueError("Event object is None")
    time = datetime.datetime.strftime(event_object.get_registration_open(), '%d/%m kl %H:%M')
    startTime = datetime.datetime.strftime(event_object.get_event_time(), '%d/%m kl: %H:%M')
    embed=discord.Embed(title=event_object.get_name(), url=event_object.get_url(), description=event_object.get_description(), color=0xff0000)
    embed.set_thumbnail(url=event_object.get_thumbnail())
    embed.add_field(name="Registrering", value=time, inline=True)
    embed.add_field(name="Når",value=startTime, inline=True)
    embed.add_field(name="Sted",value=event_object.get_event_location(),inline=True)
    return embed


def get_event_properties(message, template) -> Event:
    with open("./src/cogs/abakus/templates/"+template, "r") as f:
        pattern = f.read()
    messageSearch = re.search(pattern, message.content)
    if messageSearch:
        name = messageSearch.group(1)
        description = messageSearch.group(2)
        startTime = messageSearch.group(3)
        startTime = local_timezone.localize(datetime.datetime.strptime(startTime, '%Y-%m-%d %H:%M:%S'))
        location = messageSearch.group(4)
        signupTime = messageSearch.group(5)
        signupTime = local_timezone.localize(datetime.datetime.strptime(signupTime, '%Y-%m-%d %H:%M:%S'))
        url = messageSearch.group(6)
        event_options = {
            "name": name,
            "description": description,
            "registration_open": signupTime,
            "event_location": location,
            "event_time": startTime,
            "url": url
        }
        event_object = Event(**event_options)
        return event_object
    else:
        return